{% import "c++/_macros.hpp" as m %}

// Forward declarations, including incomplete types
{% for s in items.abstract_structs -%}
struct {{ s.name }};
{% endfor -%}
{% for s in items.structs -%}
struct {{ s.name }};
{% endfor %}

// Definitions of complete types
{% for s in items.structs %}
{%- call m::docs("", s.docs) %}
struct {{ s.name }} {
{%- for field in s.fields %}
{%- call m::docs("  ", field.docs) %}
  {{ field.ty|ty }} {{ field.name }};
{%- endfor %}
};

{% endfor %}
