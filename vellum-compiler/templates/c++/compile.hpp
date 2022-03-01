#pragma once

#include <cstdint>
#include <cstddef>
#include <utility>
#include <vellum.hpp>

{%- macro comma() %}{% if !loop.last %}, {% endif %}{% endmacro %}
{%- macro vellum_args() %}{% for arg in f.args %}{{ arg.1|ty }} vellum_arg_{{ loop.index }}{% call comma() %}{% endfor %}{% endmacro %}
{%- macro docs(indent, docs) %}
{% if !docs.is_empty() -%}
{{ indent }}/*!
{%- for doc in docs.iter() %}
{{ indent }} *{{ doc }}
{%- endfor %}
{{ indent }} */
{%- endif -%}
{%- endmacro %}

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
{{ f.returns|ty }} {{ f.name }}(
{%- for arg in f.args %}
  {{ arg.1|ty }} {{ arg.0 }}{% call comma() %}
{%- endfor %}
) noexcept;
{% endfor %}
}

{% for f in items.functions %}
#define vellum_define_{{ f.name }}({% for arg in f.args %}vellum_arg_{{ loop.index }}{% if !loop.last %}, {% endif %}{% endfor %}) \
  {{ f.returns|ty }} vellum_implement_{{ f.name }}({% call vellum_args() %}); \
  extern "C" { \
  {{ f.returns|ty }} {{ f.name }}({% call vellum_args() %}) noexcept { \
    return vellum_implement_{{ f.name }}( \
{%- for arg in f.args %}
        std::move(vellum_arg_{{loop.index}}){% call comma() %} \
{%- endfor %}
    ); \
  } \
  } \
  {{ f.returns|ty }} vellum_implement_{{ f.name }}({% call vellum_args() %})
{% endfor %}
