/**
 * @file tree-sitter parser for TsukuyomiDMX's DSL
 * @author taichi765 <taichi0209.youtub@gmail.com>
 * @license MIT
 */

/// <reference types="tree-sitter-cli/dsl" />
// @ts-check

export default grammar({
  name: "tsukuyomidmx",

  rules: {
    // TODO: add the actual grammar rules
    source_file: ($) =>
      repeat($._definition),

    _definition: ($) =>
      choice(
        $.import_statement,
        $.const_declaration,
        $.module_declaration,
        $.expression,
      ),

    import_statement: ($) =>
      seq(
        "import",
        "{",
        repeat(seq($.identifier, ",")),
        seq(
          $.identifier,
          optional(","),
        ),
        "}",
        "from",
        $.string_literal,
        ";",
      ),

    const_declaration: ($) =>
      seq(
        "const",
        $.identifier,
        "=",
        $.expression,
        ";",
      ),

    module_declaration: ($) =>
      seq(
        "module",
        $.identifier,
        $.parameters,
      ),

    parameters: ($) =>
      seq(
        "(",
        sepBy(
          ",",
          seq(
            $.identifier,
            ":",
            $._type,
          ),
        ),
        ")",
      ),

    block: ($) =>
      seq(
        "{",
        repeat($.statement),
        optional($.expression),
        "}",
      ),

    expression: ($) =>
      choice(
        $.identifier,
        $.digits,
        $.string_literal,
      ),

    statement: ($) => choice("0"),

    _type: ($) =>
      choice("bool", "color"),

    identifier: ($) => /[a-z]+/,
    string_literal: ($) =>
      seq('"', repeat(/./), '"'),
    digits: ($) => /\d+/,
  },
});

/**
 * Creates a rule to match one or more of the rules separated by the separator.
 *
 * @param {RuleOrLiteral} sep - The separator to use.
 * @param {RuleOrLiteral} rule
 *
 * @returns {SeqRule}
 */
function sepBy1(sep, rule) {
  return seq(
    rule,
    repeat(seq(sep, rule)),
  );
}

/**
 * Creates a rule to optionally match one or more of the rules separated by the separator.
 *
 * @param {RuleOrLiteral} sep - The separator to use.
 * @param {RuleOrLiteral} rule
 *
 * @returns {ChoiceRule}
 */
function sepBy(sep, rule) {
  return optional(sepBy1(sep, rule));
}
