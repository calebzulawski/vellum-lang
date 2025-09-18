{% import "c++/_macros.hpp" as m %}

// Visibility control
// - Compiling with the VELLUM_DYNAMIC macro defined enables visibility control.
// - Compiling with the VELLUM_EXPORT macro defined indicates the API is
//   being built into a shared library, rather than imported.
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

// Forward declarations
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
