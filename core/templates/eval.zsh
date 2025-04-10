print -s {{ command }};

{%- if let Some(cd) = self.cd %}
cd {{ cd }}
{% endif %}
