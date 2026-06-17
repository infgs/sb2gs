use std::sync::OnceLock;

use indexmap::IndexMap;
use regex::Regex;
use serde_json::{Map, Value, json};

use crate::json_ext::{
    array, bool_key, f64_key, key, key_mut, object, object_mut, opt_key, string, string_key,
};
use crate::signatures::{expr_signature, stmt_signature};
use crate::syntax::{self, Identifiers};

const DEFAULT_VOLUME: f64 = 100.0;
const DEFAULT_X: f64 = 0.0;
const DEFAULT_Y: f64 = 0.0;
const DEFAULT_SIZE: f64 = 100.0;
const DEFAULT_DIRECTION: f64 = 90.0;

pub struct Ctx<'a> {
    target: Value,
    assets: &'a IndexMap<String, String>,
    syntax: &'a mut Identifiers,
    out: String,
    indent_width: usize,
    indent_level: usize,
}

impl<'a> Ctx<'a> {
    pub fn new(
        target: Value,
        assets: &'a IndexMap<String, String>,
        syntax: &'a mut Identifiers,
    ) -> Self {
        Self {
            target,
            assets,
            syntax,
            out: String::new(),
            indent_width: 4,
            indent_level: 0,
        }
    }

    pub fn finish(self) -> String {
        self.out
    }

    fn print(&mut self, text: &str) {
        self.out.push_str(text);
    }

    fn println(&mut self, text: &str) {
        self.out.push_str(text);
        self.out.push('\n');
    }

    fn iprint(&mut self, text: &str) {
        self.out
            .push_str(&" ".repeat(self.indent_level * self.indent_width));
        self.out.push_str(text);
    }

    fn iprintln(&mut self, text: &str) {
        self.iprint(text);
        self.out.push('\n');
    }

    fn with_indent(&mut self, callback: impl FnOnce(&mut Self)) {
        self.indent_level += 1;
        callback(self);
        self.indent_level -= 1;
    }

    fn comma_sep<T>(&mut self, items: &[T], mut callback: impl FnMut(&mut Self, &T)) {
        for (index, item) in items.iter().enumerate() {
            callback(self, item);
            if index != items.len() - 1 {
                self.print(", ");
            }
        }
    }

    fn print_identifier(&mut self, original: &str) {
        let identifier = self.syntax.identifier(original);
        self.print(&identifier);
    }

    fn is_stage(&self) -> bool {
        bool_key(&self.target, "isStage")
    }

    fn blocks(&self) -> &Map<String, Value> {
        object(key(&self.target, "blocks"))
    }

    fn blocks_mut(&mut self) -> &mut Map<String, Value> {
        object_mut(key_mut(&mut self.target, "blocks"))
    }

    fn block(&self, id: &str) -> &Value {
        self.blocks()
            .get(id)
            .unwrap_or_else(|| panic!("missing block {id}"))
    }

    fn block_mut(&mut self, id: &str) -> &mut Value {
        self.blocks_mut()
            .get_mut(id)
            .unwrap_or_else(|| panic!("missing block {id}"))
    }
}

pub fn decompile_sprite(ctx: &mut Ctx<'_>) {
    transform(ctx);
    decompile_properties(ctx);
    decompile_costumes(ctx);
    decompile_sounds(ctx);
    decompile_variables(ctx);
    decompile_lists(ctx);
    decompile_events(ctx);
}

fn decompile_constexpr(ctx: &mut Ctx<'_>, value: &Value) {
    if let Some(boolean) = value.as_bool() {
        ctx.print(if boolean { "\"true\"" } else { "\"false\"" });
        return;
    }
    if value.is_number() {
        ctx.print(&syntax::number(value));
        return;
    }
    if let Some(text) = value.as_str() {
        ctx.print(&syntax::string(text));
        return;
    }
    panic!("unsupported value {value:?}");
}

fn decompile_asset(ctx: &mut Ctx<'_>, asset: &Value) {
    let md5ext = string_key(asset, "md5ext");
    let name = ctx
        .assets
        .get(md5ext)
        .unwrap_or_else(|| panic!("missing asset {md5ext}"))
        .clone();
    ctx.print(&syntax::string(&format!("assets/{name}")));
    if path_stem(&name) != string_key(asset, "name") {
        ctx.print(" as ");
        ctx.print(&syntax::string(string_key(asset, "name")));
    }
}

fn path_stem(name: &str) -> &str {
    let filename = name.rsplit('/').next().unwrap_or(name);
    filename.rsplit_once('.').map_or(filename, |(stem, _)| stem)
}

fn decompile_properties(ctx: &mut Ctx<'_>) {
    decompile_common_properties(ctx);
    if !ctx.is_stage() {
        decompile_sprite_properties(ctx);
    }
}

fn decompile_common_properties(ctx: &mut Ctx<'_>) {
    if f64_key(&ctx.target, "volume") != DEFAULT_VOLUME {
        ctx.iprint("set_volume ");
        ctx.print(&syntax::number(key(&ctx.target, "volume")));
        ctx.println(";");
    }
}

fn decompile_sprite_properties(ctx: &mut Ctx<'_>) {
    if !bool_key(&ctx.target, "visible") {
        ctx.iprintln("hide;");
    }
    if f64_key(&ctx.target, "x") != DEFAULT_X {
        ctx.iprint("set_x ");
        ctx.print(&syntax::number(key(&ctx.target, "x")));
        ctx.println(";");
    }
    if f64_key(&ctx.target, "y") != DEFAULT_Y {
        ctx.iprint("set_y ");
        ctx.print(&syntax::number(key(&ctx.target, "y")));
        ctx.println(";");
    }
    if f64_key(&ctx.target, "size") != DEFAULT_SIZE {
        ctx.iprint("set_size ");
        ctx.print(&syntax::number(key(&ctx.target, "size")));
        ctx.println(";");
    }
    if f64_key(&ctx.target, "direction") != DEFAULT_DIRECTION {
        ctx.iprint("point_in_direction ");
        ctx.print(&syntax::number(key(&ctx.target, "direction")));
        ctx.println(";");
    }
    decompile_rotation_style(ctx);
    if bool_key(&ctx.target, "draggable") {
        ctx.iprintln("set_draggable;");
    }
}

fn decompile_rotation_style(ctx: &mut Ctx<'_>) {
    let rotation_style = string_key(&ctx.target, "rotationStyle").to_string();
    if rotation_style == "left-right" {
        ctx.iprintln("set_rotation_style_left_right;");
    }
    if rotation_style == "don't rotate" {
        ctx.iprintln("set_rotation_style_do_not_rotate;");
    }
}

fn decompile_costumes(ctx: &mut Ctx<'_>) {
    let costumes = array(key(&ctx.target, "costumes")).to_vec();
    if costumes.is_empty() {
        return;
    }
    ctx.iprint("costumes ");
    ctx.comma_sep(&costumes, decompile_asset);
    ctx.println(";");
}

fn decompile_sounds(ctx: &mut Ctx<'_>) {
    let sounds = array(key(&ctx.target, "sounds")).to_vec();
    if sounds.is_empty() {
        return;
    }
    ctx.iprint("sounds ");
    ctx.comma_sep(&sounds, decompile_asset);
    ctx.println(";");
}

fn decompile_variables(ctx: &mut Ctx<'_>) {
    let variables = object(key(&ctx.target, "variables"))
        .values()
        .cloned()
        .collect::<Vec<_>>();
    for variable in variables {
        let parts = array(&variable);
        ctx.iprint("var ");
        ctx.print_identifier(string(&parts[0]));
        ctx.print(" = ");
        decompile_constexpr(ctx, &parts[1]);
        ctx.println(";");
    }
}

fn decompile_lists(ctx: &mut Ctx<'_>) {
    let lists = object(key(&ctx.target, "lists"))
        .values()
        .cloned()
        .collect::<Vec<_>>();
    for list in lists {
        let parts = array(&list);
        ctx.iprint("list ");
        ctx.print_identifier(string(&parts[0]));
        let values = array(&parts[1]);
        if values.is_empty() {
            ctx.println(";");
            continue;
        }
        ctx.print(" = [");
        ctx.comma_sep(values, decompile_constexpr);
        ctx.println("];");
    }
}

fn decompile_events(ctx: &mut Ctx<'_>) {
    let blocks = ctx
        .blocks()
        .values()
        .filter(|block| !block.is_array())
        .cloned()
        .collect::<Vec<_>>();
    for block in blocks {
        if bool_key(&block, "topLevel") {
            decompile_event(ctx, &block);
        }
    }
}

fn decompile_event(ctx: &mut Ctx<'_>, block: &Value) {
    match string_key(block, "opcode") {
        "event_whenflagclicked" => decompile_event_whenflagclicked(ctx, block),
        "event_whenbroadcastreceived" => decompile_event_whenbroadcastreceived(ctx, block),
        "event_whenkeypressed" => decompile_event_whenkeypressed(ctx, block),
        "control_start_as_clone" => decompile_control_start_as_clone(ctx, block),
        "event_whenthisspriteclicked" => decompile_event_whenthisspriteclicked(ctx, block),
        "procedures_definition" => decompile_procedures_definition(ctx, block),
        opcode if known_non_event_opcode(opcode) => {}
        opcode => eprintln!("no decompiler implemented for event `{opcode}`\n{block:?}"),
    }
}

fn known_non_event_opcode(opcode: &str) -> bool {
    stmt_signature(opcode).is_some()
        || expr_signature(opcode).is_some()
        || operator(opcode).is_some()
        || menu(opcode).is_some()
        || stmt_special_exists(opcode)
        || expr_special_exists(opcode)
}

fn decompile_event_whenflagclicked(ctx: &mut Ctx<'_>, block: &Value) {
    ctx.iprint("onflag ");
    decompile_stack(ctx, opt_string_key(block, "next"));
}

fn decompile_event_whenbroadcastreceived(ctx: &mut Ctx<'_>, block: &Value) {
    ctx.iprint("on ");
    ctx.print(&syntax::string(field_text(block, "BROADCAST_OPTION")));
    ctx.print(" ");
    decompile_stack(ctx, opt_string_key(block, "next"));
}

fn decompile_event_whenkeypressed(ctx: &mut Ctx<'_>, block: &Value) {
    ctx.iprint("onkey ");
    ctx.print(&syntax::string(field_text(block, "KEY_OPTION")));
    ctx.print(" ");
    decompile_stack(ctx, opt_string_key(block, "next"));
}

fn decompile_control_start_as_clone(ctx: &mut Ctx<'_>, block: &Value) {
    ctx.iprint("onclone ");
    decompile_stack(ctx, opt_string_key(block, "next"));
}

fn decompile_event_whenthisspriteclicked(ctx: &mut Ctx<'_>, block: &Value) {
    ctx.iprint("onclick ");
    decompile_stack(ctx, opt_string_key(block, "next"));
}

fn decompile_procedures_definition(ctx: &mut Ctx<'_>, block: &Value) {
    let custom_id =
        input_block_id(object(key(block, "inputs")).get("custom_block")).expect("custom block id");
    let custom = ctx.block(custom_id).clone();
    let args =
        serde_json::from_str::<Vec<String>>(string_key(key(&custom, "mutation"), "argumentnames"))
            .expect("valid argumentnames");
    if string_key(key(&custom, "mutation"), "warp") == "false" {
        ctx.iprint("nowarp proc ");
    } else {
        ctx.iprint("proc ");
    }
    ctx.print_identifier(&custom_block_name(&custom));
    if !args.is_empty() {
        ctx.print(" ");
        ctx.comma_sep(&args, |ctx, arg| ctx.print_identifier(arg));
    }
    ctx.print(" ");
    decompile_stack(ctx, opt_string_key(block, "next"));
}

fn decompile_stack(ctx: &mut Ctx<'_>, child: Option<&str>) {
    let Some(child_id) = child else {
        ctx.println("{}");
        return;
    };
    let mut child_id = child_id.to_string();
    ctx.println("{");
    ctx.with_indent(|ctx| {
        loop {
            let block = ctx.block(&child_id).clone();
            decompile_stmt(ctx, &block);
            let Some(next) = opt_string_key(&block, "next") else {
                break;
            };
            child_id = next.to_string();
        }
    });
    ctx.iprintln("}");
}

fn decompile_stmt(ctx: &mut Ctx<'_>, block: &Value) {
    if let Some((opcode, inputs)) = addon_signature(block) {
        decompile_addon(ctx, block, opcode, inputs);
        return;
    }
    let opcode = string_key(block, "opcode");
    if stmt_signature(opcode).is_some() {
        decompile_stmt_block(ctx, block);
        return;
    }
    match opcode {
        "control_if" | "control_if_else" => decompile_control_if(ctx, block),
        "control_repeat" => decompile_control_repeat(ctx, block),
        "control_repeat_until" | "control_wait_until" => decompile_control_repeat_until(ctx, block),
        "control_forever" => decompile_control_forever(ctx, block),
        "control_while" => decompile_control_while(ctx, block),
        "procedures_call" => decompile_procedures_call(ctx, block),
        "data_setvariableto" => decompile_data_setvariableto(ctx, block),
        "data_changevariableby" => decompile_data_changevariableby(ctx, block),
        "data_showvariable" => decompile_data_showvariable(ctx, block),
        "data_hidevariable" => decompile_data_hidevariable(ctx, block),
        "data_addtolist" => decompile_data_addtolist(ctx, block),
        "data_deleteoflist" => decompile_data_deleteoflist(ctx, block),
        "data_deletealloflist" => decompile_data_deletealloflist(ctx, block),
        "data_insertatlist" => decompile_data_insertatlist(ctx, block),
        "data_replaceitemoflist" => decompile_data_replaceitemoflist(ctx, block),
        "data_showlist" => decompile_data_showlist(ctx, block),
        "data_hidelist" => decompile_data_hidelist(ctx, block),
        opcode => eprintln!("no decompiler implemented for stmt `{opcode}`\n{block:?}"),
    }
}

fn decompile_stmt_block(ctx: &mut Ctx<'_>, source_block: &Value) {
    let mut block = source_block.clone();
    let mut signature = stmt_signature(string_key(&block, "opcode")).expect("stmt signature");
    apply_signature(ctx, &mut block, &mut signature);
    ctx.iprint(&signature.opcode);
    if !signature.inputs.is_empty() {
        ctx.print(" ");
        ctx.comma_sep(&signature.inputs, |ctx, input_name| {
            decompile_input(ctx, input_name, &block)
        });
    }
    ctx.println(";");
}

fn apply_signature(
    ctx: &mut Ctx<'_>,
    block: &mut Value,
    signature: &mut crate::signatures::Signature,
) {
    if let Some(menu_name) = signature.menu {
        flatten_menu(ctx, block, menu_name);
    }
    let Some(field_name) = signature.field else {
        return;
    };
    let Some(field_value) = field_text_opt(block, field_name).map(str::to_string) else {
        return;
    };
    if let Some((_, opcode)) = signature
        .overloads
        .iter()
        .find(|(key, _)| *key == field_value)
    {
        signature.opcode = (*opcode).to_string();
        signature.inputs.retain(|input| *input != field_name);
        return;
    }
    object_mut(key_mut(block, "inputs"))
        .insert(field_name.to_string(), json!([1, [4, field_value]]));
}

fn flatten_menu(ctx: &Ctx<'_>, block: &mut Value, menu_name: &str) {
    let Some(menu_id) = input_block_id(object(key(block, "inputs")).get(menu_name)) else {
        return;
    };
    let menu = ctx.block(menu_id);
    let fields = object(key(menu, "fields")).clone();
    object_mut(key_mut(block, "fields")).extend(fields);
}

fn addon_signature(block: &Value) -> Option<(&'static str, &'static [&'static str])> {
    let mutation = opt_key(block, "mutation")?;
    match opt_key(mutation, "proccode").and_then(Value::as_str)? {
        "\u{200b}\u{200b}breakpoint\u{200b}\u{200b}" => Some(("breakpoint", &[])),
        "\u{200b}\u{200b}log\u{200b}\u{200b} %s" => Some(("log", &["arg0"])),
        "\u{200b}\u{200b}warn\u{200b}\u{200b} %s" => Some(("warn", &["arg0"])),
        "\u{200b}\u{200b}error\u{200b}\u{200b} %s" => Some(("error", &["arg0"])),
        _ => None,
    }
}

fn decompile_addon(ctx: &mut Ctx<'_>, block: &Value, opcode: &str, inputs: &[&str]) {
    ctx.iprint(opcode);
    if !inputs.is_empty() {
        ctx.print(" ");
        ctx.comma_sep(inputs, |ctx, input_name| {
            decompile_input(ctx, input_name, block)
        });
    }
    ctx.println(";");
}

fn decompile_else(ctx: &mut Ctx<'_>, block_id: Option<&str>) {
    let Some(block_id) = block_id else {
        return;
    };
    let block = ctx.block(block_id).clone();
    if string_key(&block, "opcode").starts_with("control_if")
        && opt_string_key(&block, "next").is_none()
    {
        ctx.iprint("elif ");
        decompile_input(ctx, "CONDITION", &block);
        ctx.print(" ");
        decompile_stack(
            ctx,
            input_block_id(object(key(&block, "inputs")).get("SUBSTACK")),
        );
        decompile_else(
            ctx,
            input_block_id(object(key(&block, "inputs")).get("SUBSTACK2")),
        );
    } else {
        ctx.iprint("else ");
        decompile_stack(ctx, Some(block_id));
    }
}

fn decompile_control_if(ctx: &mut Ctx<'_>, block: &Value) {
    ctx.iprint("if ");
    decompile_input(ctx, "CONDITION", block);
    ctx.print(" ");
    decompile_stack(
        ctx,
        input_block_id(object(key(block, "inputs")).get("SUBSTACK")),
    );
    decompile_else(
        ctx,
        input_block_id(object(key(block, "inputs")).get("SUBSTACK2")),
    );
}

fn decompile_control_repeat(ctx: &mut Ctx<'_>, block: &Value) {
    ctx.iprint("repeat ");
    decompile_input(ctx, "TIMES", block);
    ctx.print(" ");
    decompile_stack(
        ctx,
        input_block_id(object(key(block, "inputs")).get("SUBSTACK")),
    );
}

fn decompile_control_repeat_until(ctx: &mut Ctx<'_>, block: &Value) {
    ctx.iprint("until ");
    decompile_input(ctx, "CONDITION", block);
    ctx.print(" ");
    decompile_stack(
        ctx,
        input_block_id(object(key(block, "inputs")).get("SUBSTACK")),
    );
}

fn decompile_control_forever(ctx: &mut Ctx<'_>, block: &Value) {
    ctx.iprint("forever ");
    decompile_stack(
        ctx,
        input_block_id(object(key(block, "inputs")).get("SUBSTACK")),
    );
}

fn decompile_control_while(ctx: &mut Ctx<'_>, block: &Value) {
    let condition = input_block_id(object(key(block, "inputs")).get("CONDITION"));
    if let Some(condition) = condition {
        ctx.iprint("until not ");
        let block = json!({"opcode": "operator_not", "inputs": {"OPERAND": [2, condition]}});
        decompile_operand(ctx, "OPERAND", &block, Assoc::Left);
        ctx.print(" ");
    } else {
        ctx.iprint("until not true ");
    }
    decompile_stack(
        ctx,
        input_block_id(object(key(block, "inputs")).get("SUBSTACK")),
    );
}

fn decompile_procedures_call(ctx: &mut Ctx<'_>, block: &Value) {
    ctx.iprint("");
    ctx.print_identifier(&custom_block_name(block));
    let args =
        serde_json::from_str::<Vec<String>>(string_key(key(block, "mutation"), "argumentids"))
            .expect("valid argumentids");
    if !args.is_empty() {
        ctx.print(" ");
        ctx.comma_sep(&args, |ctx, arg| decompile_input(ctx, arg, block));
    }
    ctx.println(";");
}

fn decompile_data_setvariableto(ctx: &mut Ctx<'_>, block: &Value) {
    ctx.iprint("");
    ctx.print_identifier(field_text(block, "VARIABLE"));
    ctx.print(" = ");
    decompile_input(ctx, "VALUE", block);
    ctx.println(";");
}

fn decompile_data_changevariableby(ctx: &mut Ctx<'_>, block: &Value) {
    let op = opt_key(block, "OPERATOR")
        .and_then(Value::as_str)
        .unwrap_or("+");
    ctx.iprint("");
    ctx.print_identifier(field_text(block, "VARIABLE"));
    ctx.print(&format!(" {op}= "));
    decompile_input(ctx, "VALUE", block);
    ctx.println(";");
}

fn decompile_data_showvariable(ctx: &mut Ctx<'_>, block: &Value) {
    ctx.iprint("show ");
    ctx.print_identifier(field_text(block, "VARIABLE"));
    ctx.println(";");
}

fn decompile_data_hidevariable(ctx: &mut Ctx<'_>, block: &Value) {
    ctx.iprint("hide ");
    ctx.print_identifier(field_text(block, "VARIABLE"));
    ctx.println(";");
}

fn decompile_data_addtolist(ctx: &mut Ctx<'_>, block: &Value) {
    ctx.iprint("add ");
    decompile_input(ctx, "ITEM", block);
    ctx.print(" to ");
    ctx.print_identifier(field_text(block, "LIST"));
    ctx.println(";");
}

fn decompile_data_deleteoflist(ctx: &mut Ctx<'_>, block: &Value) {
    ctx.print("delete ");
    ctx.print_identifier(field_text(block, "LIST"));
    ctx.print("[");
    decompile_input(ctx, "INDEX", block);
    ctx.println("];");
}

fn decompile_data_deletealloflist(ctx: &mut Ctx<'_>, block: &Value) {
    ctx.iprint("delete ");
    ctx.print_identifier(field_text(block, "LIST"));
    ctx.println(";");
}

fn decompile_data_insertatlist(ctx: &mut Ctx<'_>, block: &Value) {
    ctx.iprint("insert ");
    decompile_input(ctx, "ITEM", block);
    ctx.print(" at ");
    ctx.print_identifier(field_text(block, "LIST"));
    ctx.print("[");
    decompile_input(ctx, "INDEX", block);
    ctx.println("];");
}

fn decompile_data_replaceitemoflist(ctx: &mut Ctx<'_>, block: &Value) {
    let op = opt_key(block, "OPERATOR")
        .and_then(Value::as_str)
        .unwrap_or("");
    ctx.iprint("");
    ctx.print_identifier(field_text(block, "LIST"));
    ctx.print("[");
    decompile_input(ctx, "INDEX", block);
    ctx.print(&format!("] {op}= "));
    decompile_input(ctx, "ITEM", block);
    ctx.println(";");
}

fn decompile_data_showlist(ctx: &mut Ctx<'_>, block: &Value) {
    ctx.iprint("show ");
    ctx.print_identifier(field_text(block, "LIST"));
    ctx.println(";");
}

fn decompile_data_hidelist(ctx: &mut Ctx<'_>, block: &Value) {
    ctx.iprint("hide ");
    ctx.print_identifier(field_text(block, "LIST"));
    ctx.println(";");
}

fn decompile_input(ctx: &mut Ctx<'_>, input_name: &str, block: &Value) {
    let input = object(key(block, "inputs")).get(input_name);
    if input.is_none_or(is_false_input) {
        ctx.print("false");
        return;
    }
    let input = input.expect("checked input");
    if let Some(block_id) = input_block_id(Some(input)) {
        let block = ctx.block(block_id).clone();
        decompile_expr(ctx, &block);
        return;
    }
    let data = array(&array(input)[1]);
    let input_type = data[0].as_i64().expect("input type");
    let input_value = &data[1];
    if input_type == 12 || input_type == 13 {
        ctx.print_identifier(string(input_value));
        return;
    }
    ctx.print(&syntax::value(input_value));
}

fn decompile_expr(ctx: &mut Ctx<'_>, block: &Value) {
    let opcode = string_key(block, "opcode");
    if menu(opcode).is_some() {
        decompile_menu(ctx, block);
    } else if operator(opcode).is_some() && !unreal_opcode(opcode) {
        decompile_binary_operator(ctx, block);
    } else if expr_signature(opcode).is_some() {
        decompile_expr_block(ctx, block);
    } else {
        decompile_expr_special(ctx, block);
    }
}

fn decompile_expr_special(ctx: &mut Ctx<'_>, block: &Value) {
    match string_key(block, "opcode") {
        "operator_not" => decompile_operator_not(ctx, block),
        "operator_negative" => decompile_operator_negative(ctx, block),
        "operator_letter_of" => decompile_operator_letter_of(ctx, block),
        "sensing_of" => decompile_sensing_of(ctx, block),
        "data_itemoflist" => decompile_data_itemoflist(ctx, block),
        "data_itemnumoflist" => decompile_data_itemnumoflist(ctx, block),
        "data_lengthoflist" => decompile_data_lengthoflist(ctx, block),
        "argument_reporter_string_number" | "argument_reporter_boolean" => {
            decompile_argument_reporter_string_number(ctx, block)
        }
        opcode => eprintln!("no decompiler implemented for expr `{opcode}`\n{block:?}"),
    }
}

fn expr_special_exists(opcode: &str) -> bool {
    matches!(
        opcode,
        "operator_not"
            | "operator_negative"
            | "operator_letter_of"
            | "sensing_of"
            | "data_itemoflist"
            | "data_itemnumoflist"
            | "data_lengthoflist"
            | "argument_reporter_string_number"
            | "argument_reporter_boolean"
    )
}

fn stmt_special_exists(opcode: &str) -> bool {
    matches!(
        opcode,
        "control_if"
            | "control_if_else"
            | "control_repeat"
            | "control_repeat_until"
            | "control_wait_until"
            | "control_forever"
            | "control_while"
            | "procedures_call"
            | "data_setvariableto"
            | "data_changevariableby"
            | "data_showvariable"
            | "data_hidevariable"
            | "data_addtolist"
            | "data_deleteoflist"
            | "data_deletealloflist"
            | "data_insertatlist"
            | "data_replaceitemoflist"
            | "data_showlist"
            | "data_hidelist"
    )
}

fn decompile_expr_block(ctx: &mut Ctx<'_>, source_block: &Value) {
    let mut block = source_block.clone();
    let mut signature = expr_signature(string_key(&block, "opcode")).expect("expr signature");
    apply_signature(ctx, &mut block, &mut signature);
    ctx.print(&signature.opcode);
    ctx.print("(");
    if !signature.inputs.is_empty() {
        ctx.comma_sep(&signature.inputs, |ctx, input_name| {
            decompile_input(ctx, input_name, &block)
        });
    }
    ctx.print(")");
}

fn decompile_menu(ctx: &mut Ctx<'_>, block: &Value) {
    let (input_name, is_input) = menu(string_key(block, "opcode")).expect("menu");
    let data = if is_input {
        key(block, "inputs")
    } else {
        key(block, "fields")
    };
    ctx.print(&syntax::string(string(&array(key(data, input_name))[0])));
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Assoc {
    Left,
    Right,
}

#[derive(Clone, Copy)]
struct Operator {
    symbol: &'static str,
    precedence: u8,
    left_name: &'static str,
    right_name: &'static str,
    assoc: Assoc,
}

fn operator(opcode: &str) -> Option<Operator> {
    match opcode {
        "operator_letter_of" => Some(Operator {
            symbol: "",
            precedence: 1,
            left_name: "LETTER",
            right_name: "STRING",
            assoc: Assoc::Left,
        }),
        "operator_not" => Some(Operator {
            symbol: "not",
            precedence: 1,
            left_name: "OPERAND",
            right_name: "",
            assoc: Assoc::Left,
        }),
        "operator_negative" => Some(Operator {
            symbol: "-",
            precedence: 1,
            left_name: "",
            right_name: "NUM2",
            assoc: Assoc::Right,
        }),
        "operator_multiply" => Some(Operator {
            symbol: "*",
            precedence: 2,
            left_name: "NUM1",
            right_name: "NUM2",
            assoc: Assoc::Left,
        }),
        "operator_divide" => Some(Operator {
            symbol: "/",
            precedence: 2,
            left_name: "NUM1",
            right_name: "NUM2",
            assoc: Assoc::Left,
        }),
        "operator_mod" => Some(Operator {
            symbol: "%",
            precedence: 2,
            left_name: "NUM1",
            right_name: "NUM2",
            assoc: Assoc::Left,
        }),
        "operator_add" => Some(Operator {
            symbol: "+",
            precedence: 3,
            left_name: "NUM1",
            right_name: "NUM2",
            assoc: Assoc::Left,
        }),
        "operator_subtract" => Some(Operator {
            symbol: "-",
            precedence: 3,
            left_name: "NUM1",
            right_name: "NUM2",
            assoc: Assoc::Left,
        }),
        "operator_lt" => Some(Operator {
            symbol: "<",
            precedence: 4,
            left_name: "OPERAND1",
            right_name: "OPERAND2",
            assoc: Assoc::Left,
        }),
        "operator_le" => Some(Operator {
            symbol: "<=",
            precedence: 4,
            left_name: "OPERAND1",
            right_name: "OPERAND2",
            assoc: Assoc::Left,
        }),
        "operator_gt" => Some(Operator {
            symbol: ">",
            precedence: 4,
            left_name: "OPERAND1",
            right_name: "OPERAND2",
            assoc: Assoc::Left,
        }),
        "operator_ge" => Some(Operator {
            symbol: ">=",
            precedence: 4,
            left_name: "OPERAND1",
            right_name: "OPERAND2",
            assoc: Assoc::Left,
        }),
        "operator_join" => Some(Operator {
            symbol: "&",
            precedence: 5,
            left_name: "STRING1",
            right_name: "STRING2",
            assoc: Assoc::Left,
        }),
        "operator_contains" => Some(Operator {
            symbol: "in",
            precedence: 6,
            left_name: "STRING2",
            right_name: "STRING1",
            assoc: Assoc::Left,
        }),
        "data_itemnumoflist" => Some(Operator {
            symbol: "in",
            precedence: 6,
            left_name: "",
            right_name: "",
            assoc: Assoc::Left,
        }),
        "operator_equals" => Some(Operator {
            symbol: "==",
            precedence: 6,
            left_name: "OPERAND1",
            right_name: "OPERAND2",
            assoc: Assoc::Left,
        }),
        "operator_notequals" => Some(Operator {
            symbol: "!=",
            precedence: 6,
            left_name: "OPERAND1",
            right_name: "OPERAND2",
            assoc: Assoc::Left,
        }),
        "operator_and" => Some(Operator {
            symbol: "and",
            precedence: 7,
            left_name: "OPERAND1",
            right_name: "OPERAND2",
            assoc: Assoc::Left,
        }),
        "operator_or" => Some(Operator {
            symbol: "or",
            precedence: 8,
            left_name: "OPERAND1",
            right_name: "OPERAND2",
            assoc: Assoc::Left,
        }),
        _ => None,
    }
}

fn menu(opcode: &str) -> Option<(&'static str, bool)> {
    match opcode {
        "looks_costume" => Some(("COSTUME", false)),
        "sensing_of_object_menu" => Some(("OBJECT", false)),
        _ => None,
    }
}

fn unreal_opcode(opcode: &str) -> bool {
    matches!(
        opcode,
        "operator_letter_of" | "operator_not" | "operator_negative" | "data_itemnumoflist"
    )
}

fn is_parenthesis_required(parent_op: Operator, child_op: Operator, assoc: Assoc) -> bool {
    if child_op.precedence > parent_op.precedence {
        return true;
    }
    child_op.precedence == parent_op.precedence && parent_op.assoc != assoc
}

fn decompile_operand(ctx: &mut Ctx<'_>, op_name: &str, block: &Value, assoc: Assoc) {
    let op = operator(string_key(block, "opcode")).expect("operator");
    let mut parenthesis = false;
    if let Some(operand_id) = input_block_id(object(key(block, "inputs")).get(op_name)) {
        let operand_block = ctx.block(operand_id);
        parenthesis = operator(string_key(operand_block, "opcode"))
            .is_some_and(|operand_op| is_parenthesis_required(op, operand_op, assoc));
    }
    if parenthesis {
        ctx.print("(");
    }
    decompile_input(ctx, op_name, block);
    if parenthesis {
        ctx.print(")");
    }
}

fn decompile_operator_not(ctx: &mut Ctx<'_>, block: &Value) {
    ctx.print("not ");
    decompile_operand(ctx, "OPERAND", block, Assoc::Left);
}

fn decompile_operator_negative(ctx: &mut Ctx<'_>, block: &Value) {
    ctx.print("-");
    decompile_operand(ctx, "NUM2", block, Assoc::Left);
}

fn decompile_operator_letter_of(ctx: &mut Ctx<'_>, block: &Value) {
    let op = operator(string_key(block, "opcode")).expect("operator");
    decompile_operand(ctx, op.right_name, block, Assoc::Left);
    ctx.print("[");
    decompile_input(ctx, op.left_name, block);
    ctx.print("]");
}

fn decompile_binary_operator(ctx: &mut Ctx<'_>, block: &Value) {
    let op = operator(string_key(block, "opcode")).expect("operator");
    decompile_operand(ctx, op.left_name, block, Assoc::Left);
    ctx.print(" ");
    ctx.print(op.symbol);
    ctx.print(" ");
    decompile_operand(ctx, op.right_name, block, Assoc::Right);
}

fn decompile_sensing_of(ctx: &mut Ctx<'_>, block: &Value) {
    decompile_input(ctx, "OBJECT", block);
    ctx.print(".");
    ctx.print(&syntax::string(field_text(block, "PROPERTY")));
}

fn decompile_data_itemoflist(ctx: &mut Ctx<'_>, block: &Value) {
    ctx.print_identifier(field_text(block, "LIST"));
    ctx.print("[");
    decompile_input(ctx, "INDEX", block);
    ctx.print("]");
}

fn decompile_data_itemnumoflist(ctx: &mut Ctx<'_>, block: &Value) {
    decompile_operand(ctx, "ITEM", block, Assoc::Left);
    ctx.print(" in ");
    ctx.print_identifier(field_text(block, "LIST"));
}

fn decompile_data_lengthoflist(ctx: &mut Ctx<'_>, block: &Value) {
    ctx.print("length(");
    ctx.print_identifier(field_text(block, "LIST"));
    ctx.print(")");
}

fn decompile_argument_reporter_string_number(ctx: &mut Ctx<'_>, block: &Value) {
    ctx.print("$");
    ctx.print_identifier(field_text(block, "VALUE"));
}

fn transform(ctx: &mut Ctx<'_>) {
    let ids = ctx
        .blocks()
        .iter()
        .filter(|(_, block)| !block.is_array())
        .map(|(id, _)| id.clone())
        .collect::<Vec<_>>();
    for id in ids {
        transform_block(ctx, &id);
    }
}

fn transform_block(ctx: &mut Ctx<'_>, id: &str) {
    transform_list_contains_to_item_num(ctx, id);
    transform_subtract_zero_to_negative(ctx, id);
    transform_change_variable_by_negative(ctx, id);
    transform_augmented_set_variable(ctx, id);
    transform_augmented_set_variable_join(ctx, id);
    transform_augmented_replace_list_item(ctx, id);
    transform_augmented_replace_list_item_join(ctx, id);
}

fn transform_list_contains_to_item_num(ctx: &mut Ctx<'_>, id: &str) {
    if string_key(ctx.block(id), "opcode") == "data_listcontainsitem" {
        object_mut(ctx.block_mut(id)).insert(
            "opcode".to_string(),
            Value::String("data_itemnumoflist".to_string()),
        );
    }
}

fn transform_subtract_zero_to_negative(ctx: &mut Ctx<'_>, id: &str) {
    let block = ctx.block(id).clone();
    if string_key(&block, "opcode") == "operator_subtract"
        && input_block_value(object(key(&block, "inputs")).get("NUM1")).is_some_and(is_zeroish)
    {
        object_mut(ctx.block_mut(id)).insert(
            "opcode".to_string(),
            Value::String("operator_negative".to_string()),
        );
    }
}

fn transform_change_variable_by_negative(ctx: &mut Ctx<'_>, id: &str) {
    let block = ctx.block(id).clone();
    if string_key(&block, "opcode") != "data_changevariableby" {
        return;
    }
    let Some(operand) = input_block(ctx, &block, "VALUE") else {
        return;
    };
    let opcode = string_key(&operand, "opcode");
    if (opcode == "operator_subtract" || opcode == "operator_negative")
        && input_block_value(object(key(&operand, "inputs")).get("NUM1")).is_some_and(is_zeroish)
    {
        object_mut(ctx.block_mut(id))
            .insert("OPERATOR".to_string(), Value::String("-".to_string()));
        let value = object(key(&operand, "inputs"))
            .get("NUM2")
            .expect("NUM2")
            .clone();
        object_mut(key_mut(ctx.block_mut(id), "inputs")).insert("VALUE".to_string(), value);
    }
}

fn transform_augmented_set_variable(ctx: &mut Ctx<'_>, id: &str) {
    let block = ctx.block(id).clone();
    if string_key(&block, "opcode") != "data_setvariableto" {
        return;
    }
    let Some(operand) = input_block(ctx, &block, "VALUE") else {
        return;
    };
    let Some(op) = operator(string_key(&operand, "opcode")) else {
        return;
    };
    if !arithmetic_opcode(string_key(&operand, "opcode"))
        || input_variable(object(key(&operand, "inputs")).get("NUM1"))
            != Some(field_text(&block, "VARIABLE"))
    {
        return;
    }
    object_mut(ctx.block_mut(id)).insert(
        "opcode".to_string(),
        Value::String("data_changevariableby".to_string()),
    );
    object_mut(ctx.block_mut(id))
        .insert("OPERATOR".to_string(), Value::String(op.symbol.to_string()));
    let value = object(key(&operand, "inputs"))
        .get("NUM2")
        .expect("NUM2")
        .clone();
    object_mut(key_mut(ctx.block_mut(id), "inputs")).insert("VALUE".to_string(), value);
}

fn transform_augmented_set_variable_join(ctx: &mut Ctx<'_>, id: &str) {
    let block = ctx.block(id).clone();
    if string_key(&block, "opcode") != "data_setvariableto" {
        return;
    }
    let Some(operand) = input_block(ctx, &block, "VALUE") else {
        return;
    };
    if string_key(&operand, "opcode") != "operator_join"
        || input_variable(object(key(&operand, "inputs")).get("STRING1"))
            != Some(field_text(&block, "VARIABLE"))
    {
        return;
    }
    object_mut(ctx.block_mut(id)).insert(
        "opcode".to_string(),
        Value::String("data_changevariableby".to_string()),
    );
    object_mut(ctx.block_mut(id)).insert("OPERATOR".to_string(), Value::String("&".to_string()));
    let value = object(key(&operand, "inputs"))
        .get("STRING2")
        .expect("STRING2")
        .clone();
    object_mut(key_mut(ctx.block_mut(id), "inputs")).insert("VALUE".to_string(), value);
}

fn transform_augmented_replace_list_item(ctx: &mut Ctx<'_>, id: &str) {
    let block = ctx.block(id).clone();
    if string_key(&block, "opcode") != "data_replaceitemoflist" {
        return;
    }
    let Some(operand) = input_block(ctx, &block, "ITEM") else {
        return;
    };
    let Some(op) = operator(string_key(&operand, "opcode")) else {
        return;
    };
    let Some(lhs) = input_block(ctx, &operand, "NUM1") else {
        return;
    };
    if !arithmetic_opcode(string_key(&operand, "opcode"))
        || string_key(&lhs, "opcode") != "data_itemoflist"
        || field_text(&lhs, "LIST") != field_text(&block, "LIST")
        || !compare_inputs(
            ctx,
            object(key(&block, "inputs")).get("INDEX"),
            object(key(&lhs, "inputs")).get("INDEX"),
        )
    {
        return;
    }
    object_mut(ctx.block_mut(id))
        .insert("OPERATOR".to_string(), Value::String(op.symbol.to_string()));
    let value = object(key(&operand, "inputs"))
        .get("NUM2")
        .expect("NUM2")
        .clone();
    object_mut(key_mut(ctx.block_mut(id), "inputs")).insert("ITEM".to_string(), value);
}

fn transform_augmented_replace_list_item_join(ctx: &mut Ctx<'_>, id: &str) {
    let block = ctx.block(id).clone();
    if string_key(&block, "opcode") != "data_replaceitemoflist" {
        return;
    }
    let Some(operand) = input_block(ctx, &block, "ITEM") else {
        return;
    };
    let Some(lhs) = input_block(ctx, &operand, "STRING1") else {
        return;
    };
    if string_key(&operand, "opcode") != "operator_join"
        || string_key(&lhs, "opcode") != "data_itemoflist"
        || field_text(&lhs, "LIST") != field_text(&block, "LIST")
        || !compare_tree(
            ctx,
            input_block(ctx, &block, "INDEX").as_ref(),
            input_block(ctx, &lhs, "INDEX").as_ref(),
        )
    {
        return;
    }
    object_mut(ctx.block_mut(id)).insert("OPERATOR".to_string(), Value::String("&".to_string()));
    let value = object(key(&operand, "inputs"))
        .get("STRING2")
        .expect("STRING2")
        .clone();
    object_mut(key_mut(ctx.block_mut(id), "inputs")).insert("ITEM".to_string(), value);
}

fn arithmetic_opcode(opcode: &str) -> bool {
    matches!(
        opcode,
        "operator_add"
            | "operator_subtract"
            | "operator_multiply"
            | "operator_divide"
            | "operator_mod"
    )
}

fn compare_inputs(ctx: &Ctx<'_>, input1: Option<&Value>, input2: Option<&Value>) -> bool {
    let input1_block_id = input_block_id(input1);
    let input2_block_id = input_block_id(input2);
    let tree1 = input1_block_id.map(|id| ctx.block(id));
    let tree2 = input2_block_id.map(|id| ctx.block(id));
    if !compare_tree(ctx, tree1, tree2) {
        return false;
    }
    input_block_value(input1) == input_block_value(input2)
}

fn compare_tree(ctx: &Ctx<'_>, node1: Option<&Value>, node2: Option<&Value>) -> bool {
    let (Some(node1), Some(node2)) = (node1, node2) else {
        return node1.is_none() && node2.is_none();
    };
    if string_key(node1, "opcode") != string_key(node2, "opcode") {
        return false;
    }
    if object(key(node1, "fields")) != object(key(node2, "fields")) {
        return false;
    }
    for (input_name, input1_value) in object(key(node1, "inputs")) {
        let input2_value = object(key(node2, "inputs")).get(input_name);
        if input2_value.is_none() || !compare_inputs(ctx, Some(input1_value), input2_value) {
            return false;
        }
    }
    true
}

fn input_block(ctx: &Ctx<'_>, block: &Value, input_name: &str) -> Option<Value> {
    let id = input_block_id(object(key(block, "inputs")).get(input_name))?;
    Some(ctx.block(id).clone())
}

fn input_block_id(input: Option<&Value>) -> Option<&str> {
    let input = input?;
    let values = input.as_array()?;
    values.get(1)?.as_str()
}

fn input_block_value(input: Option<&Value>) -> Option<&Value> {
    let input = input?;
    let values = input.as_array()?;
    let data = values.get(1)?.as_array()?;
    data.get(1)
}

fn input_variable(input: Option<&Value>) -> Option<&str> {
    let input = input?;
    let values = input.as_array()?;
    let data = values.get(1)?.as_array()?;
    if data.first()?.as_i64()? != 12 {
        return None;
    }
    data.get(1)?.as_str()
}

fn is_zeroish(value: &Value) -> bool {
    value
        .as_str()
        .is_some_and(|value| matches!(value, "" | "0" | "0.0"))
}

fn is_false_input(input: &Value) -> bool {
    let values = input.as_array();
    values.is_some_and(|values| {
        values.len() == 2 && values[0].as_i64() == Some(1) && values[1].is_null()
    })
}

fn field_text<'a>(block: &'a Value, field_name: &str) -> &'a str {
    field_text_opt(block, field_name).unwrap_or_else(|| panic!("missing field {field_name}"))
}

fn field_text_opt<'a>(block: &'a Value, field_name: &str) -> Option<&'a str> {
    object(key(block, "fields"))
        .get(field_name)
        .and_then(|field| array(field).first())
        .and_then(Value::as_str)
}

fn opt_string_key<'a>(value: &'a Value, name: &str) -> Option<&'a str> {
    opt_key(value, name).and_then(Value::as_str)
}

fn custom_block_name(block: &Value) -> String {
    let proccode = string_key(key(block, "mutation"), "proccode");
    proccode_arg_re()
        .replace_all(proccode, "")
        .trim()
        .to_string()
}

fn proccode_arg_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r"%[sb]").expect("valid proccode regex"))
}
