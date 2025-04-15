builtin history append "{{ command }}";
builtin history merge;

{%- if let Some(cd) = self.cd %}
cd {{ cd }}
{% endif %}
