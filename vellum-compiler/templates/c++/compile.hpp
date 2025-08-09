#pragma once

#include <cstdint>
#include <cstddef>
#include <utility>
#include <vellum.hpp>

{%- macro comma() %}{% if !loop.last %}, {% endif %}{% endmacro -%}

{% macro docs(indent, docs) %}
  {%- if !docs.is_empty() %}
    {{~ indent }}/*!
  {%- for doc in docs.iter() %}
    {{~ indent }} *{{ doc }}
  {%- endfor %}
    {{~ indent }} */
  {%- endif %}
{%- endmacro %}

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

{% for s in items.abstract_structs -%}
struct {{ s.name }};
{% endfor -%}
{% for s in items.structs -%}
struct {{ s.name }};
{% endfor %}

{% for s in items.structs %}
{%- call docs("", s.docs) %}
struct {{ s.name }} {
{%- for field in s.fields %}
{%- call docs("  ", field.docs) %}
  {{ field.ty|ty }} {{ field.name }};
{%- endfor %}
};

{% endfor %}

extern "C" {

{% for f in items.functions %}
{%- call docs("", f.docs) %}
VELLUM_API {{ f.returns|retty }} {{ f.name }}(
{%- for arg in f.args %}
  {{ arg.1|ty }} {{ arg.0 }}{% call comma() %}
{%- endfor %}
) noexcept;
{% endfor %}

#define VELLUM_IMPLEMENT() \
{% for f in items.functions %} \
{{ f.returns|retty }} {{ f.name }}( \
{%- for arg in f.args %}
  {{ arg.1|ty }} {{ arg.0 }}{% call comma() %} \
{%- endfor %}
) noexcept { \
  static_assert(std::is_same_v<decltype(vellum_implement::{{ f.name}}), decltype({{ f.name }})>, \
                "vellum_implement_{{ f.name }} has incorrect signature"); \
  return vellum_implement::{{ f.name }}( \
{%- for arg in f.args %}
    std::move({{ arg.0 }}){% call comma() %} \
{%- endfor %}
  ); \
} \
{% endfor %}

}
