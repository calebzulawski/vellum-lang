{# Commonly used template macros for C++ backend #}

{%- macro comma() %}{% if !loop.last %}, {% endif %}{% endmacro -%}

{%- macro docs(indent, docs) %}
  {%- if !docs.is_empty() %}
    {{~ indent }}/*!
  {%- for doc in docs.iter() %}
    {{~ indent }} *{{ doc }}
  {%- endfor %}
    {{~ indent }} */
  {%- endif %}
{%- endmacro %}

