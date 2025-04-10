builtin history append "{{ command }}";
{%- if success %}
builtin history merge;
{%- endif %}

{%- if let Some(cd) = self.cd %}
cd {{ cd }}
{% endif %}
