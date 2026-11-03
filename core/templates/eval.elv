{%- if !command.is_empty() %}
store:add-cmd '{{ command }}'
{%- endif %}
{%- if let Some(cd) = self.cd %}
cd '{{ cd }}'
{%- endif %}
