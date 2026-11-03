use pay_respects_rule_engine::{
	Context, Mode, NoFunctions, NoScripts, RuleSet, ScriptHandler, ScriptSpec, evaluate_rule,
	parse_str,
};

fn evaluate(
	rule: &str,
	mode: Mode,
	command: &str,
	error: &str,
	executables: &[String],
) -> Vec<String> {
	let rule: RuleSet = parse_str(rule).unwrap();
	let context = Context {
		shell: "bash",
		shell_type: "bash",
		last_command: command,
		error_msg: error,
		executables,
	};
	let mut scripts = NoScripts;
	evaluate_rule(&rule, &context, mode, &mut NoFunctions, &mut scripts)
}

struct MockScripts;

impl ScriptHandler for MockScripts {
	fn evaluate(&mut self, script: &ScriptSpec, _context: &Context) -> Vec<String> {
		vec![format!("{} {}", script.interpreter, script.file)]
	}
}

#[test]
fn parses_v1_and_expands_aliases() {
	let rule = parse_str(
		r#"
command = "cat"
extends = ["bat", "batcat"]

[[match_err]]
pattern = ["No such file or directory"]
suggest = ["echo missing"]
"#,
	)
	.unwrap();

	assert_eq!(rule.version, 1);
	assert_eq!(rule.commands, vec!["cat", "bat", "batcat"]);
	assert_eq!(rule.matches.len(), 1);
}

#[test]
fn matches_patterns_without_case_or_whitespace() {
	let candidates = evaluate(
		r#"
command = "x"
[[match_err]]
pattern = ["No Such   Command"]
suggest = ["fixed"]
"#,
		Mode::Suggestion,
		"x",
		"no such command",
		&[],
	);

	assert_eq!(candidates, vec!["fixed"]);
}

#[test]
fn evaluates_command_slices() {
	let candidates = evaluate(
		r#"
version = 2
commands = ["x"]

[[match]]
[[match.suggest]]
run = "{{command}}|{{command[0]}}|{{command[-1]}}|{{command[1:]}}|{{command[:1]}}"
"#,
		Mode::Suggestion,
		"git status foo",
		"",
		&[],
	);

	assert_eq!(
		candidates,
		vec!["git status foo|git|foo|status foo|git status"]
	);
}

#[test]
fn evaluates_err_and_cmd_regex() {
	let candidates = evaluate(
		r#"
command = "git"
[[match_err]]
suggest = [
'''
{{command[0]}} {{err::(?:The most similar command is )(\S+)}} {{cmd::(?:--)(\w+)}}'''
]
"#,
		Mode::Suggestion,
		"git --all stauts",
		"The most similar command is status",
		&[],
	);

	assert_eq!(candidates, vec!["git status all"]);
}

#[test]
fn preprocesses_command_before_conditions_and_suggestions() {
	let candidates = evaluate(
		r#"
version = 2
commands = ["x"]

[[match]]
preprocess = [
	'replace("a", "b")',
	'replacen("b", "c", 1)',
]

[[match.suggest]]
when = ["cmd_contains(^x c b$)"]
run = "{{command}}"
"#,
		Mode::Suggestion,
		"x a a",
		"",
		&[],
	);

	assert_eq!(candidates, vec!["x c b"]);
}

#[test]
fn preprocess_arguments_support_quotes_commas_and_escapes() {
	let candidates = evaluate(
		r#"
version = 2
commands = ["x"]

[[match]]
preprocess = [
	'replace("a, b", "c\"d")',
]

[[match.suggest]]
run = "{{command}}"
"#,
		Mode::Suggestion,
		"x a, b",
		"",
		&[],
	);

	assert_eq!(candidates, vec!["x c\"d"]);
}

#[test]
fn evaluates_command_succeeds() {
	let candidates = evaluate(
		r#"
version = 2
commands = ["x"]

[[match]]
[[match.suggest]]
when = ['command_succeeds(test -n "{{command[0]}}")']
run = "ok"
"#,
		Mode::Suggestion,
		"x",
		"",
		&[],
	);

	assert_eq!(candidates, vec!["ok"]);

	let candidates = evaluate(
		r#"
version = 2
commands = ["x"]

[[match]]
[[match.suggest]]
when = ["command_succeeds(false)"]
run = "bad"
"#,
		Mode::Suggestion,
		"x",
		"",
		&[],
	);

	assert!(candidates.is_empty());
}

#[test]
fn reports_invalid_preprocess() {
	let cases = [
		(r#"preprocess = ['append("a", "b")']"#, "unknown preprocess"),
		(r#"preprocess = ['replace("a")']"#, "expects 2 arguments"),
		(
			r#"preprocess = ['replacen("a", "b", nope)']"#,
			"invalid replacen count",
		),
		(r#"preprocess = ['replace("a, "b")']"#, "unclosed quote"),
	];

	for (preprocess, expected) in cases {
		let rule = format!(
			r#"
version = 2
commands = ["x"]

[[match]]
{}

[[match.suggest]]
run = "x"
"#,
			preprocess
		);
		let error = parse_str(&rule).unwrap_err();
		assert!(
			error.to_string().contains(expected),
			"{}: {}",
			preprocess,
			error
		);
	}
}

#[test]
fn evaluates_select() {
	let candidates = evaluate(
		r#"
version = 2
commands = ["x"]

[[match]]
[[match.suggest]]
run = "git {{select(one, two)}} {{selection}}"
"#,
		Mode::Suggestion,
		"x",
		"",
		&[],
	);

	assert_eq!(candidates, vec!["git one one", "git two two"]);
}

#[test]
fn evaluates_opt_and_changes_later_slices() {
	let candidates = evaluate(
		r#"
command = "cp"
[[match_err]]
suggest = [
'''
{{command[0]}} {{opt::(?:\s)(-[\w]+)}} {{command[1:]}}'''
]
"#,
		Mode::Suggestion,
		"cp -r a b",
		"",
		&[],
	);

	assert_eq!(candidates, vec!["cp -r a b"]);
}

#[test]
fn inline_ignores_patterns_and_error_conditions() {
	let rule = r#"
command = "x"
[[match_err]]
pattern = ["never matches"]
suggest = [
'''
#[INLINE, err_contains(this is ignored)]
inline fix'''
]
"#;
	let candidates = evaluate(rule, Mode::Inline, "x", "no match", &[]);
	assert_eq!(candidates, vec!["inline fix"]);

	let candidates = evaluate(rule, Mode::Suggestion, "x", "no match", &[]);
	assert!(candidates.is_empty());
}

#[test]
fn function_suggestions_use_the_callback() {
	let rule = parse_str(
		r#"
command = "x"
[[match_err]]
suggest = [
'''
#[FUNCTION, min_length(1)]
MyFunction'''
]
"#,
	)
	.unwrap();
	let context = Context {
		shell: "bash",
		shell_type: "bash",
		last_command: "x",
		error_msg: "",
		executables: &[],
	};
	let mut functions = |name: &str, _context: &Context| vec![format!("called {}", name)];
	let mut scripts = NoScripts;
	let candidates = evaluate_rule(
		&rule,
		&context,
		Mode::Suggestion,
		&mut functions,
		&mut scripts,
	);

	assert_eq!(candidates, vec!["called MyFunction"]);
}

#[test]
fn parses_v2_suggestions() {
	let rule = parse_str(
		r#"
version = 2
commands = ["x"]

[[match]]
error = ["failed"]

[[match.suggest]]
when = ["length(1)"]
inline = true
run = "fixed"
"#,
	)
	.unwrap();

	assert_eq!(rule.version, 2);
	assert_eq!(rule.commands, vec!["x"]);
	assert_eq!(rule.matches.len(), 1);
	assert_eq!(rule.matches[0].patterns, vec!["failed"]);
	assert_eq!(rule.matches[0].suggestions.len(), 1);
}

#[test]
fn expands_v2_variables() {
	let candidates = evaluate(
		r#"
version = 2
commands = ["x"]

[[match]]
vars = { dir = "{{command[1]}}", path = "/tmp/{{$dir}}" }

[[match.suggest]]
run = "cd {{$path}}"
"#,
		Mode::Suggestion,
		"x build",
		"",
		&[],
	);

	assert_eq!(candidates, vec!["cd /tmp/build"]);
}

#[test]
fn accepts_variable_pairs() {
	let candidates = evaluate(
		r#"
version = 2
commands = ["x"]

[[match]]
vars = [
	["cmd", "{{command[0]}}"],
	["arg", "{{command[1]}}"],
]

[[match.suggest]]
run = "{{$cmd}} {{$arg}}"
"#,
		Mode::Suggestion,
		"git status",
		"",
		&[],
	);

	assert_eq!(candidates, vec!["git status"]);
}

#[test]
fn variables_work_inside_placeholders() {
	let candidates = evaluate(
		r#"
version = 2
commands = ["x"]

[[match]]
vars = { choices = "one, two" }

[[match.suggest]]
run = "{{select({{$choices}})}}"
"#,
		Mode::Suggestion,
		"x",
		"",
		&[],
	);

	assert_eq!(candidates, vec!["one", "two"]);
}

#[test]
fn evaluates_scripts_through_the_handler() {
	let rule = parse_str(
		r#"
version = 2
commands = ["x"]

[[match]]
vars = { file = "git-fix.py" }

[[match.suggest]]
script = { interpreter = "/usr/bin/python3", file = "{{$file}}" }
"#,
	)
	.unwrap();
	let context = Context {
		shell: "bash",
		shell_type: "bash",
		last_command: "x src",
		error_msg: "",
		executables: &[],
	};
	let mut functions = NoFunctions;
	let mut scripts = MockScripts;
	let candidates = evaluate_rule(
		&rule,
		&context,
		Mode::Suggestion,
		&mut functions,
		&mut scripts,
	);

	assert_eq!(candidates, vec!["/usr/bin/python3 git-fix.py"]);
}

#[test]
fn rejects_scripts_without_runtime_support() {
	let error = pay_respects_rule_engine::parse_str_with(
		r#"
version = 2
commands = ["x"]

[[match]]
[[match.suggest]]
script = { interpreter = "/bin/sh", file = "fix.sh" }
"#,
		pay_respects_rule_engine::ParseOptions {
			allow_scripts: false,
		},
	)
	.unwrap_err();

	assert!(
		error
			.to_string()
			.contains("only available in runtime rules")
	);
}

#[test]
fn reports_unknown_and_recursive_variables() {
	let error = parse_str(
		r#"
version = 2
commands = ["x"]

[[match]]
[[match.suggest]]
run = "{{$missing}}"
"#,
	)
	.unwrap_err();
	assert!(error.to_string().contains("unknown variable"));

	let error = parse_str(
		r#"
version = 2
commands = ["x"]

[[match]]
vars = { a = "{{$b}}", b = "{{$a}}" }

[[match.suggest]]
run = "{{$a}}"
"#,
	)
	.unwrap_err();
	assert!(error.to_string().contains("recursive variable"));
}

#[test]
fn migrates_v1_to_v2() {
	let v1 = r#"
command = "hello"
extends = ["hi"]

[[match_err]]
pattern = ["failed"]
suggest = [
'''
#[executable(sudo), INLINE]
sudo {{command}} ''',
'''
#[FUNCTION, min_length(2)]
MyFunction ''',
]
"#;
	let v2 = pay_respects_rule_engine::migrate_str(v1).unwrap();
	let rule = parse_str(&v2).unwrap();

	assert_eq!(rule.version, 2);
	assert_eq!(rule.commands, vec!["hello", "hi"]);
	assert_eq!(rule.matches.len(), 1);
	assert_eq!(rule.matches[0].suggestions.len(), 2);
	assert!(v2.contains("when = ["));
	assert!(v2.contains("inline = true"));
	assert!(v2.contains("function = 'MyFunction'"));
}

#[test]
fn rejects_migrating_v2() {
	let error = pay_respects_rule_engine::migrate_str(
		r#"
version = 2
commands = ["x"]
"#,
	)
	.unwrap_err();

	assert!(error.to_string().contains("already version 2"));
}

#[test]
fn reports_unknown_conditions() {
	let error = parse_str(
		r#"
command = "x"
[[match_err]]
suggest = [
'''
#[not_a_condition(value)]
x'''
]
"#,
	)
	.unwrap_err();

	assert!(error.to_string().contains("unknown condition"));
}

#[test]
fn reports_unknown_v2_fields() {
	let error = parse_str(
		r#"
version = 2
commands = ["x"]
unknown = true

[[match]]
[[match.suggest]]
run = "x"
"#,
	)
	.unwrap_err();

	assert!(error.to_string().contains("unknown field"));
}

#[test]
fn detects_executable_requirements() {
	let rule = parse_str(
		r#"
command = "x"
[[match_err]]
suggest = ["{{select(path)}}"]
"#,
	)
	.unwrap();

	assert!(rule.needs_executables());

	let rule = parse_str(
		r#"
command = "x"
[[match_err]]
suggest = ["echo fixed"]
"#,
	)
	.unwrap();

	assert!(!rule.needs_executables());
}
