{% import "c++/_macros.hpp" as m %}

#define VELLUM_IMPLEMENT() \
namespace vellum_private_abi { extern "C" { \
  {% for f in items.functions -%}
  {{ f.returns|retty }} {{ f.name }}( \
  {%- for arg in f.args %}
    {{ arg.1|ty }} {{ arg.0 }}{% call m::comma() %} \
  {%- endfor %}
  ) noexcept { \
    using vellum_expected_signature__{{ f.name }} = {{ f.returns|retty_raii }} (*)( \
    {%- for arg in f.args %}
      {{ arg.1|ty_raii }}{% call m::comma() %} \
    {%- endfor %}
    ) noexcept; \
    static_assert(std::is_same_v<decltype(&vellum_implement::{{ f.name }}), vellum_expected_signature__{{ f.name }}>, \
                  "vellum_implement::{{ f.name }} has incorrect signature"); \
    {%- if f.returns.is_some() %}
    return \
    {%- else %}
    (void) \
    {%- endif %}
    vellum_implement::{{ f.name }}( \
    {%- for arg in f.args %}
      std::move({{ arg.0 }}){% call m::comma() %} \
    {%- endfor %}
    ); \
  } \
{% endfor -%}
} }
