{% import "c++/_macros.hpp" as m %}

#define VELLUM_IMPLEMENT() \
 \
{% for f in items.functions -%}
{{ f.returns|retty }} {{ f.name }}( \
{%- for arg in f.args %}
  {{ arg.1|ty }} {{ arg.0 }}{% call m::comma() %} \
{%- endfor %}
) noexcept { \
  static_assert(std::is_same_v<decltype(vellum_implement::{{ f.name}}), decltype({{ f.name }})>, \
                "vellum_implement::{{ f.name }} has incorrect signature"); \
  return vellum_implement::{{ f.name }}( \
{%- for arg in f.args %}
    std::move({{ arg.0 }}){% call m::comma() %} \
{%- endfor %}
  ); \
} {% if !loop.last %}\{% endif %}
{% endfor -%}
