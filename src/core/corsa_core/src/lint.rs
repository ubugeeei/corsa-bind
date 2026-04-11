use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::utils::{
    TypeTextKind, classify_type_text, is_array_like_type_texts, is_error_like_type_texts,
    is_number_like_type_texts, is_promise_like_type_texts, is_string_like_type_texts,
    split_type_text,
};

/// UTF-8 byte range used by Oxlint-compatible rule diagnostics and fixes.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TextRange {
    pub start: u32,
    pub end: u32,
}

impl TextRange {
    pub const fn new(start: u32, end: u32) -> Self {
        Self { start, end }
    }

    pub const fn is_empty(self) -> bool {
        self.start == self.end
    }

    pub const fn is_valid(self) -> bool {
        self.start <= self.end
    }
}

/// Serializable AST/type facts passed from the Oxlint JS plugin boundary into
/// Rust-authored rules.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LintNode {
    pub kind: String,
    pub range: TextRange,
    #[serde(default)]
    pub text: Option<String>,
    #[serde(default)]
    pub type_texts: Vec<String>,
    #[serde(default)]
    pub property_names: Vec<String>,
    #[serde(default)]
    pub fields: BTreeMap<String, Value>,
    #[serde(default)]
    pub children: BTreeMap<String, LintNode>,
    #[serde(default)]
    pub child_lists: BTreeMap<String, Vec<LintNode>>,
}

impl LintNode {
    pub fn child(&self, key: &str) -> Option<&Self> {
        self.children.get(key)
    }

    pub fn child_list(&self, key: &str) -> Option<&[Self]> {
        self.child_lists.get(key).map(Vec::as_slice)
    }

    pub fn field_str(&self, key: &str) -> Option<&str> {
        self.fields.get(key).and_then(Value::as_str)
    }

    pub fn field_bool(&self, key: &str) -> Option<bool> {
        self.fields.get(key).and_then(Value::as_bool)
    }

    pub fn field_f64(&self, key: &str) -> Option<f64> {
        self.fields.get(key).and_then(Value::as_f64)
    }

    pub fn field_stringish(&self, key: &str) -> Option<String> {
        self.fields.get(key).and_then(|value| match value {
            Value::String(value) => Some(value.clone()),
            Value::Number(value) => Some(value.to_string()),
            Value::Bool(value) => Some(value.to_string()),
            _ => None,
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LintFix {
    pub range: TextRange,
    pub replacement_text: String,
}

impl LintFix {
    pub fn replace_range(range: TextRange, replacement_text: impl Into<String>) -> Self {
        Self {
            range,
            replacement_text: replacement_text.into(),
        }
    }

    pub fn remove_range(range: TextRange) -> Self {
        Self::replace_range(range, "")
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LintSuggestion {
    pub message_id: String,
    pub message: String,
    pub fixes: Vec<LintFix>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LintDiagnostic {
    pub rule_name: String,
    pub message_id: String,
    pub message: String,
    pub range: TextRange,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub suggestions: Vec<LintSuggestion>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RuleMessage {
    pub id: &'static str,
    pub description: &'static str,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RuleMeta {
    pub name: String,
    pub docs_description: String,
    pub messages: BTreeMap<String, String>,
    pub has_suggestions: bool,
    pub listeners: Vec<String>,
    pub requires_type_texts: bool,
}

/// A Rust-authored type-aware lint rule.
///
/// The host adapter owns AST traversal and type lookups, then sends compact
/// [`LintNode`] facts into the Rust rule. This keeps the final public surface as
/// an Oxlint JS plugin while allowing common rules to live on the Rust hot path.
pub trait RustLintRule: Send + Sync {
    fn name(&self) -> &'static str;

    fn docs_description(&self) -> &'static str;

    fn messages(&self) -> &'static [RuleMessage];

    fn listeners(&self) -> &'static [&'static str];

    fn has_suggestions(&self) -> bool {
        false
    }

    fn requires_type_texts(&self) -> bool {
        true
    }

    fn check(&self, ctx: &mut RuleContext<'_>, node: &LintNode);

    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: self.name().to_owned(),
            docs_description: self.docs_description().to_owned(),
            messages: self
                .messages()
                .iter()
                .map(|message| (message.id.to_owned(), message.description.to_owned()))
                .collect(),
            has_suggestions: self.has_suggestions(),
            listeners: self
                .listeners()
                .iter()
                .map(|listener| (*listener).to_owned())
                .collect(),
            requires_type_texts: self.requires_type_texts(),
        }
    }
}

pub struct RuleContext<'a> {
    rule: &'a dyn RustLintRule,
    diagnostics: Vec<LintDiagnostic>,
}

impl<'a> RuleContext<'a> {
    fn new(rule: &'a dyn RustLintRule) -> Self {
        Self {
            rule,
            diagnostics: Vec::new(),
        }
    }

    pub fn report(&mut self, message_id: &'static str, range: TextRange) {
        self.report_with_suggestions(message_id, range, Vec::new());
    }

    pub fn report_with_suggestions(
        &mut self,
        message_id: &'static str,
        range: TextRange,
        suggestions: Vec<LintSuggestion>,
    ) {
        self.diagnostics.push(LintDiagnostic {
            rule_name: self.rule.name().to_owned(),
            message_id: message_id.to_owned(),
            message: self.message(message_id),
            range,
            suggestions,
        });
    }

    fn finish(self) -> Vec<LintDiagnostic> {
        self.diagnostics
    }

    fn message(&self, message_id: &str) -> String {
        self.rule
            .messages()
            .iter()
            .find(|message| message.id == message_id)
            .map(|message| message.description)
            .unwrap_or(message_id)
            .to_owned()
    }
}

#[derive(Default)]
pub struct LintRuleRegistry {
    rules: Vec<Box<dyn RustLintRule>>,
}

impl LintRuleRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_rule(mut self, rule: impl RustLintRule + 'static) -> Self {
        self.rules.push(Box::new(rule));
        self
    }

    pub fn with_default_type_aware_rules() -> Self {
        Self::new()
            .with_rule(NoArrayDeleteRule)
            .with_rule(NoForInArrayRule)
            .with_rule(AwaitThenableRule)
            .with_rule(NoImpliedEvalRule)
            .with_rule(NoMixedEnumsRule)
            .with_rule(NoUnsafeUnaryMinusRule)
            .with_rule(OnlyThrowErrorRule)
            .with_rule(PreferFindRule)
            .with_rule(PreferIncludesRule)
            .with_rule(PreferRegexpExecRule)
            .with_rule(UseUnknownInCatchCallbackVariableRule)
    }

    pub fn rule_names(&self) -> impl Iterator<Item = &'static str> + '_ {
        self.rules.iter().map(|rule| rule.name())
    }

    pub fn metas(&self) -> Vec<RuleMeta> {
        self.rules.iter().map(|rule| rule.meta()).collect()
    }

    pub fn run_rule(&self, rule_name: &str, node: &LintNode) -> Option<Vec<LintDiagnostic>> {
        let rule = self
            .rules
            .iter()
            .find(|candidate| candidate.name() == rule_name)?;
        let mut ctx = RuleContext::new(rule.as_ref());
        rule.check(&mut ctx, node);
        Some(ctx.finish())
    }
}

pub fn run_default_type_aware_rule(
    rule_name: &str,
    node: &LintNode,
) -> Option<Vec<LintDiagnostic>> {
    LintRuleRegistry::with_default_type_aware_rules().run_rule(rule_name, node)
}

#[derive(Clone, Copy, Debug, Default)]
pub struct NoArrayDeleteRule;

const NO_ARRAY_DELETE_MESSAGES: &[RuleMessage] = &[
    RuleMessage {
        id: "unexpected",
        description: "Do not delete elements from an array-like value.",
    },
    RuleMessage {
        id: "useSplice",
        description: "Use array.splice(index, 1) instead.",
    },
];
const NO_ARRAY_DELETE_LISTENERS: &[&str] = &["UnaryExpression"];

impl RustLintRule for NoArrayDeleteRule {
    fn name(&self) -> &'static str {
        "no-array-delete"
    }

    fn docs_description(&self) -> &'static str {
        "Disallow deleting elements from array-like values."
    }

    fn messages(&self) -> &'static [RuleMessage] {
        NO_ARRAY_DELETE_MESSAGES
    }

    fn listeners(&self) -> &'static [&'static str] {
        NO_ARRAY_DELETE_LISTENERS
    }

    fn has_suggestions(&self) -> bool {
        true
    }

    fn check(&self, ctx: &mut RuleContext<'_>, node: &LintNode) {
        if node.kind != "UnaryExpression" || node.field_str("operator") != Some("delete") {
            return;
        }
        let Some(argument) = node.child("argument") else {
            return;
        };
        if argument.kind != "MemberExpression" || argument.field_bool("computed") != Some(true) {
            return;
        }
        let Some(object) = argument.child("object") else {
            return;
        };
        if object.kind != "ArrayExpression" && !is_array_like_type_texts(&object.type_texts) {
            return;
        }
        let suggestions = splice_suggestion(node, argument, object)
            .into_iter()
            .collect();
        ctx.report_with_suggestions("unexpected", node.range, suggestions);
    }
}

fn splice_suggestion(
    node: &LintNode,
    argument: &LintNode,
    object: &LintNode,
) -> Option<LintSuggestion> {
    let property = argument.child("property")?;
    let delete_range = TextRange::new(node.range.start, object.range.start);
    let left_bracket_range = TextRange::new(object.range.end, property.range.start);
    let right_bracket_range = TextRange::new(property.range.end, argument.range.end);
    if !delete_range.is_valid() || !left_bracket_range.is_valid() || !right_bracket_range.is_valid()
    {
        return None;
    }
    Some(LintSuggestion {
        message_id: "useSplice".to_owned(),
        message: "Use array.splice(index, 1) instead.".to_owned(),
        fixes: vec![
            LintFix::remove_range(delete_range),
            LintFix::replace_range(left_bracket_range, ".splice("),
            LintFix::replace_range(right_bracket_range, ", 1)"),
        ],
    })
}

#[derive(Clone, Copy, Debug, Default)]
pub struct NoForInArrayRule;

const NO_FOR_IN_ARRAY_MESSAGES: &[RuleMessage] = &[RuleMessage {
    id: "unexpected",
    description: "Do not iterate over an array with a for-in loop.",
}];
const NO_FOR_IN_ARRAY_LISTENERS: &[&str] = &["ForInStatement"];

impl RustLintRule for NoForInArrayRule {
    fn name(&self) -> &'static str {
        "no-for-in-array"
    }

    fn docs_description(&self) -> &'static str {
        "Disallow for-in iteration over array-like values."
    }

    fn messages(&self) -> &'static [RuleMessage] {
        NO_FOR_IN_ARRAY_MESSAGES
    }

    fn listeners(&self) -> &'static [&'static str] {
        NO_FOR_IN_ARRAY_LISTENERS
    }

    fn check(&self, ctx: &mut RuleContext<'_>, node: &LintNode) {
        if node.kind != "ForInStatement" {
            return;
        }
        let Some(right) = node.child("right") else {
            return;
        };
        if right.kind == "ArrayExpression" || is_array_like_type_texts(&right.type_texts) {
            ctx.report("unexpected", node.range);
        }
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct AwaitThenableRule;

const AWAIT_THENABLE_MESSAGES: &[RuleMessage] = &[RuleMessage {
    id: "unexpected",
    description: "Unexpected await of a non-thenable value.",
}];
const AWAIT_THENABLE_LISTENERS: &[&str] = &["AwaitExpression"];

impl RustLintRule for AwaitThenableRule {
    fn name(&self) -> &'static str {
        "await-thenable"
    }

    fn docs_description(&self) -> &'static str {
        "Disallow awaiting non-thenable values."
    }

    fn messages(&self) -> &'static [RuleMessage] {
        AWAIT_THENABLE_MESSAGES
    }

    fn listeners(&self) -> &'static [&'static str] {
        AWAIT_THENABLE_LISTENERS
    }

    fn check(&self, ctx: &mut RuleContext<'_>, node: &LintNode) {
        if node.kind != "AwaitExpression" {
            return;
        }
        let Some(argument) = node.child("argument") else {
            return;
        };
        if !is_promise_like_node(argument) && !is_obviously_promise_like(argument) {
            ctx.report("unexpected", node.range);
        }
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct NoImpliedEvalRule;

const NO_IMPLIED_EVAL_MESSAGES: &[RuleMessage] = &[RuleMessage {
    id: "unexpected",
    description: "Do not pass a string to an implied eval API.",
}];
const NO_IMPLIED_EVAL_LISTENERS: &[&str] = &["CallExpression", "NewExpression"];

impl RustLintRule for NoImpliedEvalRule {
    fn name(&self) -> &'static str {
        "no-implied-eval"
    }

    fn docs_description(&self) -> &'static str {
        "Disallow string-based dynamic code execution APIs."
    }

    fn messages(&self) -> &'static [RuleMessage] {
        NO_IMPLIED_EVAL_MESSAGES
    }

    fn listeners(&self) -> &'static [&'static str] {
        NO_IMPLIED_EVAL_LISTENERS
    }

    fn check(&self, ctx: &mut RuleContext<'_>, node: &LintNode) {
        match node.kind.as_str() {
            "CallExpression" => {
                let Some(callee) = node.child("callee") else {
                    return;
                };
                let callee = strip_chain_expression(callee);
                let callee_name = member_property_name(callee).or_else(|| identifier_name(callee));
                if !matches!(
                    callee_name.as_deref(),
                    Some("execScript" | "setInterval" | "setTimeout")
                ) {
                    return;
                }
                let Some(first_argument) = first_child_list_item(node, "arguments") else {
                    return;
                };
                if !first_argument.kind.contains("Function")
                    && (is_literal_string(first_argument)
                        || is_string_like_type_texts(&first_argument.type_texts))
                {
                    ctx.report("unexpected", node.range);
                }
            }
            "NewExpression" => {
                let Some(callee) = node.child("callee") else {
                    return;
                };
                if !is_identifier_named(callee, "Function") {
                    return;
                }
                if child_list(node, "arguments").iter().any(is_literal_string) {
                    ctx.report("unexpected", node.range);
                }
            }
            _ => {}
        }
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct NoMixedEnumsRule;

const NO_MIXED_ENUMS_MESSAGES: &[RuleMessage] = &[RuleMessage {
    id: "mixed",
    description: "Mixing number and string enums can be confusing.",
}];
const NO_MIXED_ENUMS_LISTENERS: &[&str] = &["TSEnumDeclaration"];

impl RustLintRule for NoMixedEnumsRule {
    fn name(&self) -> &'static str {
        "no-mixed-enums"
    }

    fn docs_description(&self) -> &'static str {
        "Disallow mixing string and numeric enum members."
    }

    fn messages(&self) -> &'static [RuleMessage] {
        NO_MIXED_ENUMS_MESSAGES
    }

    fn listeners(&self) -> &'static [&'static str] {
        NO_MIXED_ENUMS_LISTENERS
    }

    fn check(&self, ctx: &mut RuleContext<'_>, node: &LintNode) {
        if node.kind != "TSEnumDeclaration" {
            return;
        }
        let members = enum_members_of(node);
        let Some(first_member) = members.first() else {
            return;
        };
        let desired_kind = enum_member_kind(first_member);
        if desired_kind == EnumMemberKind::Unknown {
            return;
        }
        for member in members {
            let current_kind = enum_member_kind(member);
            if current_kind == EnumMemberKind::Unknown {
                return;
            }
            if current_kind != desired_kind {
                ctx.report(
                    "mixed",
                    member
                        .child("initializer")
                        .map(|initializer| initializer.range)
                        .unwrap_or(member.range),
                );
                return;
            }
        }
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct NoUnsafeUnaryMinusRule;

const NO_UNSAFE_UNARY_MINUS_MESSAGES: &[RuleMessage] = &[RuleMessage {
    id: "unaryMinus",
    description: "Argument of unary negation should be assignable to number | bigint.",
}];
const NO_UNSAFE_UNARY_MINUS_LISTENERS: &[&str] = &["UnaryExpression"];

impl RustLintRule for NoUnsafeUnaryMinusRule {
    fn name(&self) -> &'static str {
        "no-unsafe-unary-minus"
    }

    fn docs_description(&self) -> &'static str {
        "Disallow unary negation on non-number and non-bigint values."
    }

    fn messages(&self) -> &'static [RuleMessage] {
        NO_UNSAFE_UNARY_MINUS_MESSAGES
    }

    fn listeners(&self) -> &'static [&'static str] {
        NO_UNSAFE_UNARY_MINUS_LISTENERS
    }

    fn check(&self, ctx: &mut RuleContext<'_>, node: &LintNode) {
        if node.kind != "UnaryExpression" || node.field_str("operator") != Some("-") {
            return;
        }
        let Some(argument) = node.child("argument") else {
            return;
        };
        if is_number_or_bigint_literal(argument) || is_unary_minus_type_safe(&argument.type_texts) {
            return;
        }
        ctx.report("unaryMinus", node.range);
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct OnlyThrowErrorRule;

const ONLY_THROW_ERROR_MESSAGES: &[RuleMessage] = &[RuleMessage {
    id: "unexpected",
    description: "Only Error-like values should be thrown.",
}];
const ONLY_THROW_ERROR_LISTENERS: &[&str] = &["ThrowStatement"];

impl RustLintRule for OnlyThrowErrorRule {
    fn name(&self) -> &'static str {
        "only-throw-error"
    }

    fn docs_description(&self) -> &'static str {
        "Require thrown values to be Error-like."
    }

    fn messages(&self) -> &'static [RuleMessage] {
        ONLY_THROW_ERROR_MESSAGES
    }

    fn listeners(&self) -> &'static [&'static str] {
        ONLY_THROW_ERROR_LISTENERS
    }

    fn check(&self, ctx: &mut RuleContext<'_>, node: &LintNode) {
        if node.kind != "ThrowStatement" {
            return;
        }
        let Some(argument) = node.child("argument") else {
            return;
        };
        if !is_error_like_node(argument) {
            ctx.report("unexpected", node.range);
        }
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct PreferFindRule;

const PREFER_FIND_MESSAGES: &[RuleMessage] = &[RuleMessage {
    id: "unexpected",
    description: "Use .find() instead of filtering and taking the first match.",
}];
const PREFER_FIND_LISTENERS: &[&str] = &["CallExpression", "MemberExpression"];

impl RustLintRule for PreferFindRule {
    fn name(&self) -> &'static str {
        "prefer-find"
    }

    fn docs_description(&self) -> &'static str {
        "Prefer find over filtering and taking the first element."
    }

    fn messages(&self) -> &'static [RuleMessage] {
        PREFER_FIND_MESSAGES
    }

    fn listeners(&self) -> &'static [&'static str] {
        PREFER_FIND_LISTENERS
    }

    fn requires_type_texts(&self) -> bool {
        false
    }

    fn check(&self, ctx: &mut RuleContext<'_>, node: &LintNode) {
        match node.kind.as_str() {
            "MemberExpression" => {
                if member_property_name(node).as_deref() == Some("0")
                    && callee_property_name(node.child("object")).as_deref() == Some("filter")
                {
                    ctx.report("unexpected", node.range);
                }
            }
            "CallExpression" => {
                if callee_property_name(Some(node)).as_deref() != Some("at")
                    || !first_child_list_item(node, "arguments").is_some_and(is_zero_literal)
                {
                    return;
                }
                let Some(callee) = node.child("callee") else {
                    return;
                };
                if callee_property_name(member_object(callee)).as_deref() == Some("filter") {
                    ctx.report("unexpected", node.range);
                }
            }
            _ => {}
        }
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct PreferIncludesRule;

const PREFER_INCLUDES_MESSAGES: &[RuleMessage] = &[RuleMessage {
    id: "unexpected",
    description: "Use .includes() instead of comparing an index result.",
}];
const PREFER_INCLUDES_LISTENERS: &[&str] = &["BinaryExpression"];

impl RustLintRule for PreferIncludesRule {
    fn name(&self) -> &'static str {
        "prefer-includes"
    }

    fn docs_description(&self) -> &'static str {
        "Prefer includes over indexOf/lastIndexOf comparisons."
    }

    fn messages(&self) -> &'static [RuleMessage] {
        PREFER_INCLUDES_MESSAGES
    }

    fn listeners(&self) -> &'static [&'static str] {
        PREFER_INCLUDES_LISTENERS
    }

    fn requires_type_texts(&self) -> bool {
        false
    }

    fn check(&self, ctx: &mut RuleContext<'_>, node: &LintNode) {
        if node.kind != "BinaryExpression" {
            return;
        }
        let Some(left) = node.child("left") else {
            return;
        };
        let Some(right) = node.child("right") else {
            return;
        };
        if (is_comparable_index_search(left) || is_comparable_index_search(right))
            && (is_negative_one_literal(left)
                || is_negative_one_literal(right)
                || is_zero_literal(left)
                || is_zero_literal(right))
        {
            ctx.report("unexpected", node.range);
        }
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct PreferRegexpExecRule;

const PREFER_REGEXP_EXEC_MESSAGES: &[RuleMessage] = &[RuleMessage {
    id: "unexpected",
    description: "Use a RegExp exec() call instead of String match().",
}];
const PREFER_REGEXP_EXEC_LISTENERS: &[&str] = &["CallExpression"];

impl RustLintRule for PreferRegexpExecRule {
    fn name(&self) -> &'static str {
        "prefer-regexp-exec"
    }

    fn docs_description(&self) -> &'static str {
        "Prefer RegExp#exec over String#match for single matches."
    }

    fn messages(&self) -> &'static [RuleMessage] {
        PREFER_REGEXP_EXEC_MESSAGES
    }

    fn listeners(&self) -> &'static [&'static str] {
        PREFER_REGEXP_EXEC_LISTENERS
    }

    fn requires_type_texts(&self) -> bool {
        false
    }

    fn check(&self, ctx: &mut RuleContext<'_>, node: &LintNode) {
        if node.kind != "CallExpression"
            || callee_property_name(Some(node)).as_deref() != Some("match")
        {
            return;
        }
        let Some(first_argument) = first_child_list_item(node, "arguments") else {
            return;
        };
        let Some(flags) = regex_flags(first_argument) else {
            return;
        };
        if !flags.contains('g') {
            ctx.report("unexpected", node.range);
        }
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct UseUnknownInCatchCallbackVariableRule;

const USE_UNKNOWN_IN_CATCH_CALLBACK_VARIABLE_MESSAGES: &[RuleMessage] = &[RuleMessage {
    id: "unexpected",
    description: "Catch callback variables should be explicitly typed as unknown.",
}];
const USE_UNKNOWN_IN_CATCH_CALLBACK_VARIABLE_LISTENERS: &[&str] = &["CallExpression"];

impl RustLintRule for UseUnknownInCatchCallbackVariableRule {
    fn name(&self) -> &'static str {
        "use-unknown-in-catch-callback-variable"
    }

    fn docs_description(&self) -> &'static str {
        "Require Promise catch callback variables to use an explicit unknown annotation."
    }

    fn messages(&self) -> &'static [RuleMessage] {
        USE_UNKNOWN_IN_CATCH_CALLBACK_VARIABLE_MESSAGES
    }

    fn listeners(&self) -> &'static [&'static str] {
        USE_UNKNOWN_IN_CATCH_CALLBACK_VARIABLE_LISTENERS
    }

    fn requires_type_texts(&self) -> bool {
        false
    }

    fn check(&self, ctx: &mut RuleContext<'_>, node: &LintNode) {
        if node.kind != "CallExpression" {
            return;
        }
        let property_name = callee_property_name(Some(node));
        let callback = match property_name.as_deref() {
            Some("catch") => child_list(node, "arguments").first(),
            Some("then") => child_list(node, "arguments").get(1),
            _ => None,
        };
        let Some(callback) = callback else {
            return;
        };
        if !callback.kind.contains("Function") {
            return;
        }
        let Some(parameter) = callback
            .child_list("params")
            .and_then(|params| params.first())
        else {
            return;
        };
        if !has_unknown_type_annotation(parameter) {
            ctx.report("unexpected", parameter.range);
        }
    }
}

fn strip_chain_expression(mut node: &LintNode) -> &LintNode {
    while node.kind == "ChainExpression" {
        let Some(expression) = node.child("expression") else {
            break;
        };
        node = expression;
    }
    node
}

fn is_promise_like_node(node: &LintNode) -> bool {
    is_promise_like_type_texts(&node.type_texts, &node.property_names)
}

fn is_obviously_promise_like(node: &LintNode) -> bool {
    let current = strip_chain_expression(node);
    if current.kind == "NewExpression" {
        return current
            .child("callee")
            .is_some_and(|callee| is_identifier_named(callee, "Promise"));
    }
    if current.kind != "CallExpression" {
        return false;
    }
    let Some(callee) = current.child("callee") else {
        return false;
    };
    member_property_name(callee).as_deref() == Some("resolve")
        && member_object(callee).is_some_and(|object| is_identifier_named(object, "Promise"))
}

fn is_error_like_node(node: &LintNode) -> bool {
    let current = strip_chain_expression(node);
    if current.kind == "NewExpression"
        && current
            .child("callee")
            .and_then(|callee| identifier_name(callee).or_else(|| member_property_name(callee)))
            .is_some_and(|identifier| identifier.ends_with("Error"))
    {
        return true;
    }
    is_error_like_type_texts(&node.type_texts, &node.property_names)
}

fn member_property_name(node: &LintNode) -> Option<String> {
    let current = strip_chain_expression(node);
    if current.kind != "MemberExpression" {
        return None;
    }
    let property = current.child("property")?;
    if !current.field_bool("computed").unwrap_or(false) && property.kind == "Identifier" {
        return property.field_stringish("name");
    }
    if current.field_bool("computed").unwrap_or(false) && property.kind == "Literal" {
        return property.field_stringish("value");
    }
    None
}

fn member_object(node: &LintNode) -> Option<&LintNode> {
    let current = strip_chain_expression(node);
    if current.kind == "MemberExpression" {
        current.child("object")
    } else {
        None
    }
}

fn callee_property_name(node: Option<&LintNode>) -> Option<String> {
    let node = strip_chain_expression(node?);
    if node.kind == "CallExpression" {
        node.child("callee").and_then(member_property_name)
    } else {
        None
    }
}

fn identifier_name(node: &LintNode) -> Option<String> {
    let current = strip_chain_expression(node);
    if current.kind == "Identifier" {
        current.field_stringish("name")
    } else {
        None
    }
}

fn is_identifier_named(node: &LintNode, name: &str) -> bool {
    identifier_name(node).as_deref() == Some(name)
}

fn is_literal_string(node: &LintNode) -> bool {
    let current = strip_chain_expression(node);
    current.kind == "Literal" && current.fields.get("value").is_some_and(Value::is_string)
}

fn is_negative_one_literal(node: &LintNode) -> bool {
    let current = strip_chain_expression(node);
    if current.kind == "Literal" && current.field_f64("value") == Some(-1.0) {
        return true;
    }
    current.kind == "UnaryExpression"
        && current.field_str("operator") == Some("-")
        && current
            .child("argument")
            .is_some_and(|arg| arg.kind == "Literal" && arg.field_f64("value") == Some(1.0))
}

fn is_zero_literal(node: &LintNode) -> bool {
    let current = strip_chain_expression(node);
    current.kind == "Literal" && current.field_f64("value") == Some(0.0)
}

fn is_number_or_bigint_literal(node: &LintNode) -> bool {
    let current = strip_chain_expression(node);
    current.kind == "Literal"
        && (current.fields.get("value").is_some_and(Value::is_number)
            || current.fields.get("bigint").is_some_and(Value::is_string))
}

fn is_comparable_index_search(node: &LintNode) -> bool {
    matches!(
        callee_property_name(Some(node)).as_deref(),
        Some("indexOf" | "lastIndexOf")
    )
}

fn first_child_list_item<'a>(node: &'a LintNode, key: &str) -> Option<&'a LintNode> {
    node.child_list(key).and_then(|items| items.first())
}

fn child_list<'a>(node: &'a LintNode, key: &str) -> &'a [LintNode] {
    node.child_list(key).unwrap_or(&[])
}

fn regex_flags(node: &LintNode) -> Option<&str> {
    node.fields
        .get("regex")
        .and_then(|regex| regex.get("flags"))
        .and_then(Value::as_str)
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum EnumMemberKind {
    Number,
    String,
    Unknown,
}

fn enum_members_of(node: &LintNode) -> &[LintNode] {
    node.child("body")
        .and_then(|body| body.child_list("members"))
        .or_else(|| node.child_list("members"))
        .unwrap_or(&[])
}

fn enum_member_kind(member: &LintNode) -> EnumMemberKind {
    let Some(initializer) = member.child("initializer") else {
        return EnumMemberKind::Number;
    };
    if initializer.kind == "Literal" {
        if initializer
            .fields
            .get("value")
            .is_some_and(Value::is_number)
        {
            return EnumMemberKind::Number;
        }
        if initializer
            .fields
            .get("value")
            .is_some_and(Value::is_string)
        {
            return EnumMemberKind::String;
        }
    }
    if is_string_like_type_texts(&initializer.type_texts) {
        return EnumMemberKind::String;
    }
    if is_number_like_type_texts(&initializer.type_texts) {
        return EnumMemberKind::Number;
    }
    EnumMemberKind::Unknown
}

fn has_unknown_type_annotation(node: &LintNode) -> bool {
    node.child("typeAnnotation")
        .and_then(|type_annotation| {
            type_annotation
                .child("typeAnnotation")
                .or(Some(type_annotation))
        })
        .is_some_and(|type_annotation| type_annotation.kind == "TSUnknownKeyword")
}

fn is_unary_minus_type_safe<T: AsRef<str>>(type_texts: &[T]) -> bool {
    !type_texts.is_empty()
        && type_texts.iter().all(|text| {
            split_type_text(text.as_ref()).iter().all(|part| {
                matches!(
                    classify_type_text(Some(part.as_str())),
                    TypeTextKind::Any | TypeTextKind::Number | TypeTextKind::Bigint
                )
            })
        })
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use serde_json::json;

    use super::{LintNode, LintRuleRegistry, TextRange};

    #[test]
    fn reports_array_delete_with_splice_suggestion() {
        let diagnostics = registry()
            .run_rule("no-array-delete", &array_delete_node())
            .unwrap();

        assert_eq!(diagnostics.len(), 1);
        assert_eq!(diagnostics[0].rule_name, "no-array-delete");
        assert_eq!(diagnostics[0].message_id, "unexpected");
        assert_eq!(diagnostics[0].range, TextRange::new(0, 20));
        assert_eq!(diagnostics[0].suggestions.len(), 1);
        assert_eq!(diagnostics[0].suggestions[0].message_id, "useSplice");
        assert_eq!(diagnostics[0].suggestions[0].fixes.len(), 3);
        assert_eq!(
            diagnostics[0].suggestions[0].fixes[0].range,
            TextRange::new(0, 7)
        );
        assert_eq!(
            diagnostics[0].suggestions[0].fixes[1].range,
            TextRange::new(13, 14)
        );
        assert_eq!(
            diagnostics[0].suggestions[0].fixes[2].range,
            TextRange::new(19, 20)
        );
    }

    #[test]
    fn ignores_non_array_member_delete() {
        let mut node = array_delete_node();
        node.children
            .get_mut("argument")
            .unwrap()
            .children
            .get_mut("object")
            .unwrap()
            .type_texts = vec!["{ value: number }".to_owned()];

        let diagnostics = registry().run_rule("no-array-delete", &node).unwrap();

        assert!(diagnostics.is_empty());
    }

    #[test]
    fn reports_for_in_array() {
        let diagnostics = registry()
            .run_rule("no-for-in-array", &for_in_array_node("readonly string[]"))
            .unwrap();

        assert_eq!(diagnostics.len(), 1);
        assert_eq!(diagnostics[0].rule_name, "no-for-in-array");
        assert_eq!(diagnostics[0].message_id, "unexpected");
    }

    #[test]
    fn ignores_for_in_record() {
        let diagnostics = registry()
            .run_rule("no-for-in-array", &for_in_array_node("{ value: number }"))
            .unwrap();

        assert!(diagnostics.is_empty());
    }

    #[test]
    fn reports_for_in_array_literal() {
        let mut node = for_in_array_node("");
        node.children.get_mut("right").unwrap().kind = "ArrayExpression".to_owned();

        let diagnostics = registry().run_rule("no-for-in-array", &node).unwrap();

        assert_eq!(diagnostics.len(), 1);
    }

    #[test]
    fn lists_default_rule_meta() {
        let registry = registry();
        assert_eq!(
            registry.rule_names().collect::<Vec<_>>(),
            vec![
                "no-array-delete",
                "no-for-in-array",
                "await-thenable",
                "no-implied-eval",
                "no-mixed-enums",
                "no-unsafe-unary-minus",
                "only-throw-error",
                "prefer-find",
                "prefer-includes",
                "prefer-regexp-exec",
                "use-unknown-in-catch-callback-variable"
            ]
        );
        let metas = registry.metas();
        assert_eq!(metas[0].name, "no-array-delete");
        assert_eq!(metas[0].listeners, vec!["UnaryExpression"]);
        assert_eq!(
            metas[0].messages.get("unexpected").unwrap(),
            "Do not delete elements from an array-like value."
        );
        assert_eq!(metas[1].name, "no-for-in-array");
        assert_eq!(metas[1].listeners, vec!["ForInStatement"]);
    }

    fn registry() -> LintRuleRegistry {
        LintRuleRegistry::with_default_type_aware_rules()
    }

    fn array_delete_node() -> LintNode {
        LintNode {
            kind: "UnaryExpression".to_owned(),
            range: TextRange::new(0, 20),
            text: None,
            type_texts: Vec::new(),
            property_names: Vec::new(),
            fields: BTreeMap::from([("operator".to_owned(), json!("delete"))]),
            children: BTreeMap::from([(
                "argument".to_owned(),
                LintNode {
                    kind: "MemberExpression".to_owned(),
                    range: TextRange::new(7, 20),
                    text: None,
                    type_texts: Vec::new(),
                    property_names: Vec::new(),
                    fields: BTreeMap::from([("computed".to_owned(), json!(true))]),
                    children: BTreeMap::from([
                        (
                            "object".to_owned(),
                            LintNode {
                                kind: "Identifier".to_owned(),
                                range: TextRange::new(7, 13),
                                text: Some("values".to_owned()),
                                type_texts: vec!["number[]".to_owned()],
                                property_names: Vec::new(),
                                fields: BTreeMap::new(),
                                children: BTreeMap::new(),
                                child_lists: BTreeMap::new(),
                            },
                        ),
                        (
                            "property".to_owned(),
                            LintNode {
                                kind: "Identifier".to_owned(),
                                range: TextRange::new(14, 19),
                                text: Some("index".to_owned()),
                                type_texts: Vec::new(),
                                property_names: Vec::new(),
                                fields: BTreeMap::new(),
                                children: BTreeMap::new(),
                                child_lists: BTreeMap::new(),
                            },
                        ),
                    ]),
                    child_lists: BTreeMap::new(),
                },
            )]),
            child_lists: BTreeMap::new(),
        }
    }

    fn for_in_array_node(right_type_text: &str) -> LintNode {
        LintNode {
            kind: "ForInStatement".to_owned(),
            range: TextRange::new(0, 42),
            text: None,
            type_texts: Vec::new(),
            property_names: Vec::new(),
            fields: BTreeMap::new(),
            children: BTreeMap::from([(
                "right".to_owned(),
                LintNode {
                    kind: "Identifier".to_owned(),
                    range: TextRange::new(18, 24),
                    text: Some("values".to_owned()),
                    type_texts: vec![right_type_text.to_owned()],
                    property_names: Vec::new(),
                    fields: BTreeMap::new(),
                    children: BTreeMap::new(),
                    child_lists: BTreeMap::new(),
                },
            )]),
            child_lists: BTreeMap::new(),
        }
    }
}
