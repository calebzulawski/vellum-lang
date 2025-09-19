{% import "c++/_macros.hpp" as m %}

#ifndef VELLUM_API
  #ifdef VELLUM_DYNAMIC
    #ifdef VELLUM_EXPORT
      #if defined(_WIN32) || defined(__CYGWIN__)
        #define VELLUM_API __declspec(dllexport)
      #else
        #define VELLUM_API __attribute__((visibility("default")))
      #endif
    #else
      #if defined(_WIN32) || defined(__CYGWIN__)
        #define VELLUM_API __declspec(dllimport)
      #else
        #define VELLUM_API
      #endif
    #endif
  #else
    #define VELLUM_API
  #endif
#endif

namespace vellum_private_abi {
extern "C" {

{% for f in items.functions %}
{%- call m::docs("", f.docs) %}
VELLUM_API {{ f.returns|retty }} {{ f.name }}(
{%- for arg in f.args %}
  {{ arg.1|ty }} {{ arg.0 }}{% call m::comma() %}
{%- endfor %}
) noexcept;
{% endfor %}

}
}

{% for f in items.functions %}
{%- call m::docs("", f.docs) %}
inline {{ f.returns|retty_raii }} {{ f.name }}(
{%- for arg in f.args %}
  {{ arg.1|ty_raii }} {{ arg.0 }}{% call m::comma() %}
{%- endfor %}
) noexcept {
  {%- if f.returns.is_some() %}
  return vellum_private_abi::{{ f.name }}(
  {%- else %}
  vellum_private_abi::{{ f.name }}(
  {%- endif %}
  {%- for arg in f.args %}
    std::move({{ arg.0 }}){% call m::comma() %}
  {%- endfor %}
  );
}
{% endfor %}
