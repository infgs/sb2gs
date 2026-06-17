#[derive(Clone, Copy)]
pub(crate) struct SignatureSpec {
    pub(crate) source: &'static str,
    pub(crate) opcode: &'static str,
    pub(crate) inputs: &'static [&'static str],
    pub(crate) menu: Option<&'static str>,
    pub(crate) field: Option<&'static str>,
    pub(crate) overloads: &'static [(&'static str, &'static str)],
}

#[derive(Clone)]
pub(crate) struct Signature {
    pub(crate) opcode: String,
    pub(crate) inputs: Vec<&'static str>,
    pub(crate) menu: Option<&'static str>,
    pub(crate) field: Option<&'static str>,
    pub(crate) overloads: &'static [(&'static str, &'static str)],
}

impl SignatureSpec {
    fn to_signature(self) -> Signature {
        Signature {
            opcode: self.opcode.to_string(),
            inputs: self.inputs.to_vec(),
            menu: self.menu,
            field: self.field,
            overloads: self.overloads,
        }
    }
}

const EMPTY: &[(&str, &str)] = &[];
const MOTION_GOTO: &[(&str, &str)] = &[
    ("_mouse_", "goto_mouse_pointer"),
    ("_random_", "goto_random_position"),
];
const MOTION_GLIDETO: &[(&str, &str)] = &[
    ("_mouse_", "glide_to_mouse_pointer"),
    ("_random_", "glide_to_random_position"),
];
const MOTION_POINTTOWARDS: &[(&str, &str)] = &[
    ("_mouse_", "point_towards_mouse_pointer"),
    ("_random_", "point_towards_random_direction"),
];
const MOTION_ROTATION_STYLE: &[(&str, &str)] = &[
    ("left-right", "set_rotation_style_left_right"),
    ("don't rotate", "set_rotation_style_do_not_rotate"),
    ("all around", "set_rotation_style_all_around"),
];
const BACKDROP: &[(&str, &str)] = &[
    ("next backdrop", "next_backdrop"),
    ("previous backdrop", "previous_backdrop"),
    ("random backdrop", "random_backdrop"),
];
const LOOKS_EFFECT_CHANGE: &[(&str, &str)] = &[
    ("COLOR", "change_color_effect"),
    ("FISHEYE", "change_fisheye_effect"),
    ("WHIRL", "change_whirl_effect"),
    ("PIXELATE", "change_pixelate_effect"),
    ("MOSAIC", "change_mosaic_effect"),
    ("BRIGHTNESS", "change_brightness_effect"),
    ("GHOST", "change_ghost_effect"),
];
const LOOKS_EFFECT_SET: &[(&str, &str)] = &[
    ("COLOR", "set_color_effect"),
    ("FISHEYE", "set_fisheye_effect"),
    ("WHIRL", "set_whirl_effect"),
    ("PIXELATE", "set_pixelate_effect"),
    ("MOSAIC", "set_mosaic_effect"),
    ("BRIGHTNESS", "set_brightness_effect"),
    ("GHOST", "set_ghost_effect"),
];
const FRONT_BACK: &[(&str, &str)] = &[("front", "goto_front"), ("back", "goto_back")];
const FORWARD_BACKWARD: &[(&str, &str)] = &[("forward", "go_forward"), ("backward", "go_backward")];
const SOUND_EFFECT_CHANGE: &[(&str, &str)] = &[
    ("PITCH", "change_pitch_effect"),
    ("PAN", "change_pan_effect"),
];
const SOUND_EFFECT_SET: &[(&str, &str)] =
    &[("PITCH", "set_pitch_effect"), ("PAN", "set_pan_effect")];
const CONTROL_STOP: &[(&str, &str)] = &[
    ("all", "stop_all"),
    ("this script", "stop_this_script"),
    ("other scripts in sprite", "stop_other_scripts"),
];
const CLONE: &[(&str, &str)] = &[("_myself_", "clone")];
const DRAG_MODE: &[(&str, &str)] = &[
    ("draggable", "set_drag_mode_draggable"),
    ("not draggable", "set_drag_mode_not_draggable"),
];
const PEN_COLOR_PARAM_SET: &[(&str, &str)] = &[
    ("color", "set_pen_hue"),
    ("saturation", "set_pen_saturation"),
    ("brightness", "set_pen_brightness"),
    ("transparency", "set_pen_transparency"),
];
const PEN_COLOR_PARAM_CHANGE: &[(&str, &str)] = &[
    ("color", "change_pen_hue"),
    ("saturation", "change_pen_saturation"),
    ("brightness", "change_pen_brightness"),
    ("transparency", "change_pen_transparency"),
];
const COSTUME_NUMBER_NAME: &[(&str, &str)] =
    &[("number", "costume_number"), ("name", "costume_name")];
const BACKDROP_NUMBER_NAME: &[(&str, &str)] =
    &[("number", "backdrop_number"), ("name", "backdrop_name")];
const DISTANCE_TO: &[(&str, &str)] = &[("_mouse_", "distance_to_mouse_pointer")];
const TOUCHING: &[(&str, &str)] = &[
    ("_mouse_", "touching_mouse_pointer"),
    ("_edge_", "touching_edge"),
];
const CURRENT: &[(&str, &str)] = &[
    ("YEAR", "current_year"),
    ("MONTH", "current_month"),
    ("DATE", "current_date"),
    ("DAYOFWEEK", "current_day_of_week"),
    ("HOUR", "current_hour"),
    ("MINUTE", "current_minute"),
    ("SECOND", "current_second"),
];
const MATHOP: &[(&str, &str)] = &[
    ("abs", "abs"),
    ("floor", "floor"),
    ("ceiling", "ceil"),
    ("sqrt", "sqrt"),
    ("sin", "sin"),
    ("cos", "cos"),
    ("tan", "tan"),
    ("asin", "asin"),
    ("acos", "acos"),
    ("atan", "atan"),
    ("ln", "ln"),
    ("log", "log"),
    ("e ^", "antiln"),
    ("10 ^", "antilog"),
];

pub(crate) const STMT_SIGNATURES: &[SignatureSpec] = &[
    SignatureSpec {
        source: "motion_movesteps",
        opcode: "move",
        inputs: &["STEPS"],
        menu: None,
        field: None,
        overloads: EMPTY,
    },
    SignatureSpec {
        source: "motion_turnleft",
        opcode: "turn_left",
        inputs: &["DEGREES"],
        menu: None,
        field: None,
        overloads: EMPTY,
    },
    SignatureSpec {
        source: "motion_turnright",
        opcode: "turn_right",
        inputs: &["DEGREES"],
        menu: None,
        field: None,
        overloads: EMPTY,
    },
    SignatureSpec {
        source: "motion_goto",
        opcode: "goto",
        inputs: &["TO"],
        menu: Some("TO"),
        field: Some("TO"),
        overloads: MOTION_GOTO,
    },
    SignatureSpec {
        source: "motion_gotoxy",
        opcode: "goto",
        inputs: &["X", "Y"],
        menu: None,
        field: None,
        overloads: EMPTY,
    },
    SignatureSpec {
        source: "motion_glidesecstoxy",
        opcode: "glide",
        inputs: &["X", "Y", "SECS"],
        menu: None,
        field: None,
        overloads: EMPTY,
    },
    SignatureSpec {
        source: "motion_glideto",
        opcode: "glide",
        inputs: &["TO", "SECS"],
        menu: Some("TO"),
        field: Some("TO"),
        overloads: MOTION_GLIDETO,
    },
    SignatureSpec {
        source: "motion_pointindirection",
        opcode: "point_in_direction",
        inputs: &["DIRECTION"],
        menu: None,
        field: None,
        overloads: EMPTY,
    },
    SignatureSpec {
        source: "motion_pointtowards",
        opcode: "point_towards",
        inputs: &["TOWARDS"],
        menu: Some("TOWARDS"),
        field: Some("TOWARDS"),
        overloads: MOTION_POINTTOWARDS,
    },
    SignatureSpec {
        source: "motion_changexby",
        opcode: "change_x",
        inputs: &["DX"],
        menu: None,
        field: None,
        overloads: EMPTY,
    },
    SignatureSpec {
        source: "motion_setx",
        opcode: "set_x",
        inputs: &["X"],
        menu: None,
        field: None,
        overloads: EMPTY,
    },
    SignatureSpec {
        source: "motion_changeyby",
        opcode: "change_y",
        inputs: &["DY"],
        menu: None,
        field: None,
        overloads: EMPTY,
    },
    SignatureSpec {
        source: "motion_sety",
        opcode: "set_y",
        inputs: &["Y"],
        menu: None,
        field: None,
        overloads: EMPTY,
    },
    SignatureSpec {
        source: "motion_ifonedgebounce",
        opcode: "if_on_edge_bounce",
        inputs: &[],
        menu: None,
        field: None,
        overloads: EMPTY,
    },
    SignatureSpec {
        source: "motion_setrotationstyle",
        opcode: "set_rotation_style_all_around",
        inputs: &[],
        menu: None,
        field: Some("STYLE"),
        overloads: MOTION_ROTATION_STYLE,
    },
    SignatureSpec {
        source: "looks_sayforsecs",
        opcode: "say",
        inputs: &["MESSAGE", "SECS"],
        menu: None,
        field: None,
        overloads: EMPTY,
    },
    SignatureSpec {
        source: "looks_thinkforsecs",
        opcode: "think",
        inputs: &["MESSAGE", "SECS"],
        menu: None,
        field: None,
        overloads: EMPTY,
    },
    SignatureSpec {
        source: "looks_say",
        opcode: "say",
        inputs: &["MESSAGE"],
        menu: None,
        field: None,
        overloads: EMPTY,
    },
    SignatureSpec {
        source: "looks_think",
        opcode: "think",
        inputs: &["MESSAGE"],
        menu: None,
        field: None,
        overloads: EMPTY,
    },
    SignatureSpec {
        source: "looks_switchcostumeto",
        opcode: "switch_costume",
        inputs: &["COSTUME"],
        menu: None,
        field: None,
        overloads: EMPTY,
    },
    SignatureSpec {
        source: "looks_nextcostume",
        opcode: "next_costume",
        inputs: &[],
        menu: None,
        field: None,
        overloads: EMPTY,
    },
    SignatureSpec {
        source: "looks_switchbackdropto",
        opcode: "switch_backdrop",
        inputs: &["BACKDROP"],
        menu: Some("BACKDROP"),
        field: Some("BACKDROP"),
        overloads: BACKDROP,
    },
    SignatureSpec {
        source: "looks_nextbackdrop",
        opcode: "next_backdrop",
        inputs: &[],
        menu: None,
        field: None,
        overloads: EMPTY,
    },
    SignatureSpec {
        source: "looks_setsizeto",
        opcode: "set_size",
        inputs: &["SIZE"],
        menu: None,
        field: None,
        overloads: EMPTY,
    },
    SignatureSpec {
        source: "looks_changesizeby",
        opcode: "change_size",
        inputs: &["CHANGE"],
        menu: None,
        field: None,
        overloads: EMPTY,
    },
    SignatureSpec {
        source: "looks_changeeffectby",
        opcode: "change_color_effect",
        inputs: &["CHANGE"],
        menu: None,
        field: Some("EFFECT"),
        overloads: LOOKS_EFFECT_CHANGE,
    },
    SignatureSpec {
        source: "looks_seteffectto",
        opcode: "set_color_effect",
        inputs: &["VALUE"],
        menu: None,
        field: Some("EFFECT"),
        overloads: LOOKS_EFFECT_SET,
    },
    SignatureSpec {
        source: "looks_cleargraphiceffects",
        opcode: "clear_graphic_effects",
        inputs: &[],
        menu: None,
        field: None,
        overloads: EMPTY,
    },
    SignatureSpec {
        source: "looks_show",
        opcode: "show",
        inputs: &[],
        menu: None,
        field: None,
        overloads: EMPTY,
    },
    SignatureSpec {
        source: "looks_hide",
        opcode: "hide",
        inputs: &[],
        menu: None,
        field: None,
        overloads: EMPTY,
    },
    SignatureSpec {
        source: "looks_gotofrontback",
        opcode: "goto_front",
        inputs: &[],
        menu: None,
        field: Some("FRONT_BACK"),
        overloads: FRONT_BACK,
    },
    SignatureSpec {
        source: "looks_goforwardbackwardlayers",
        opcode: "go_forward",
        inputs: &["NUM"],
        menu: None,
        field: Some("FORWARD_BACKWARD"),
        overloads: FORWARD_BACKWARD,
    },
    SignatureSpec {
        source: "sound_playuntildone",
        opcode: "play_sound_until_done",
        inputs: &["SOUND_MENU"],
        menu: Some("SOUND_MENU"),
        field: Some("SOUND_MENU"),
        overloads: EMPTY,
    },
    SignatureSpec {
        source: "sound_play",
        opcode: "start_sound",
        inputs: &["SOUND_MENU"],
        menu: Some("SOUND_MENU"),
        field: Some("SOUND_MENU"),
        overloads: EMPTY,
    },
    SignatureSpec {
        source: "sound_stopallsounds",
        opcode: "stop_all_sounds",
        inputs: &[],
        menu: None,
        field: None,
        overloads: EMPTY,
    },
    SignatureSpec {
        source: "sound_changeeffectby",
        opcode: "change_pitch_effect",
        inputs: &["VALUE"],
        menu: None,
        field: Some("EFFECT"),
        overloads: SOUND_EFFECT_CHANGE,
    },
    SignatureSpec {
        source: "sound_seteffectto",
        opcode: "set_pitch_effect",
        inputs: &["VALUE"],
        menu: None,
        field: Some("EFFECT"),
        overloads: SOUND_EFFECT_SET,
    },
    SignatureSpec {
        source: "sound_changevolumeby",
        opcode: "change_volume",
        inputs: &["VOLUME"],
        menu: None,
        field: None,
        overloads: EMPTY,
    },
    SignatureSpec {
        source: "sound_setvolumeto",
        opcode: "set_volume",
        inputs: &["VOLUME"],
        menu: None,
        field: None,
        overloads: EMPTY,
    },
    SignatureSpec {
        source: "sound_cleareffects",
        opcode: "clear_sound_effects",
        inputs: &[],
        menu: None,
        field: None,
        overloads: EMPTY,
    },
    SignatureSpec {
        source: "event_broadcast",
        opcode: "broadcast",
        inputs: &["BROADCAST_INPUT"],
        menu: None,
        field: None,
        overloads: EMPTY,
    },
    SignatureSpec {
        source: "event_broadcastandwait",
        opcode: "broadcast_and_wait",
        inputs: &["BROADCAST_INPUT"],
        menu: None,
        field: None,
        overloads: EMPTY,
    },
    SignatureSpec {
        source: "control_wait",
        opcode: "wait",
        inputs: &["DURATION"],
        menu: None,
        field: None,
        overloads: EMPTY,
    },
    SignatureSpec {
        source: "control_stop",
        opcode: "stop_all",
        inputs: &[],
        menu: None,
        field: Some("STOP_OPTION"),
        overloads: CONTROL_STOP,
    },
    SignatureSpec {
        source: "control_delete_this_clone",
        opcode: "delete_this_clone",
        inputs: &[],
        menu: None,
        field: None,
        overloads: EMPTY,
    },
    SignatureSpec {
        source: "control_create_clone_of",
        opcode: "clone",
        inputs: &["CLONE_OPTION"],
        menu: Some("CLONE_OPTION"),
        field: Some("CLONE_OPTION"),
        overloads: CLONE,
    },
    SignatureSpec {
        source: "sensing_askandwait",
        opcode: "ask",
        inputs: &["QUESTION"],
        menu: None,
        field: None,
        overloads: EMPTY,
    },
    SignatureSpec {
        source: "sensing_setdragmode",
        opcode: "set_drag_mode_draggable",
        inputs: &[],
        menu: None,
        field: Some("DRAG_MODE"),
        overloads: DRAG_MODE,
    },
    SignatureSpec {
        source: "sensing_resettimer",
        opcode: "reset_timer",
        inputs: &[],
        menu: None,
        field: None,
        overloads: EMPTY,
    },
    SignatureSpec {
        source: "pen_clear",
        opcode: "erase_all",
        inputs: &[],
        menu: None,
        field: None,
        overloads: EMPTY,
    },
    SignatureSpec {
        source: "pen_stamp",
        opcode: "stamp",
        inputs: &[],
        menu: None,
        field: None,
        overloads: EMPTY,
    },
    SignatureSpec {
        source: "pen_penDown",
        opcode: "pen_down",
        inputs: &[],
        menu: None,
        field: None,
        overloads: EMPTY,
    },
    SignatureSpec {
        source: "pen_penUp",
        opcode: "pen_up",
        inputs: &[],
        menu: None,
        field: None,
        overloads: EMPTY,
    },
    SignatureSpec {
        source: "pen_setPenColorToColor",
        opcode: "set_pen_color",
        inputs: &["COLOR"],
        menu: None,
        field: None,
        overloads: EMPTY,
    },
    SignatureSpec {
        source: "pen_changePenSizeBy",
        opcode: "change_pen_size",
        inputs: &["SIZE"],
        menu: None,
        field: None,
        overloads: EMPTY,
    },
    SignatureSpec {
        source: "pen_setPenSizeTo",
        opcode: "set_pen_size",
        inputs: &["SIZE"],
        menu: None,
        field: None,
        overloads: EMPTY,
    },
    SignatureSpec {
        source: "pen_setPenColorParamTo",
        opcode: "set_pen_hue",
        inputs: &["VALUE"],
        menu: None,
        field: Some("COLOR_PARAM"),
        overloads: PEN_COLOR_PARAM_SET,
    },
    SignatureSpec {
        source: "pen_changePenColorParamBy",
        opcode: "change_pen_hue",
        inputs: &["VALUE"],
        menu: None,
        field: Some("COLOR_PARAM"),
        overloads: PEN_COLOR_PARAM_CHANGE,
    },
    SignatureSpec {
        source: "music_restForBeats",
        opcode: "rest",
        inputs: &["BEATS"],
        menu: None,
        field: None,
        overloads: EMPTY,
    },
    SignatureSpec {
        source: "music_setTempo",
        opcode: "set_tempo",
        inputs: &["TEMPO"],
        menu: None,
        field: None,
        overloads: EMPTY,
    },
    SignatureSpec {
        source: "music_changeTempo",
        opcode: "change_tempo",
        inputs: &["TEMPO"],
        menu: None,
        field: None,
        overloads: EMPTY,
    },
];

pub(crate) const EXPR_SIGNATURES: &[SignatureSpec] = &[
    SignatureSpec {
        source: "motion_xposition",
        opcode: "x_position",
        inputs: &[],
        menu: None,
        field: None,
        overloads: EMPTY,
    },
    SignatureSpec {
        source: "motion_yposition",
        opcode: "y_position",
        inputs: &[],
        menu: None,
        field: None,
        overloads: EMPTY,
    },
    SignatureSpec {
        source: "motion_direction",
        opcode: "direction",
        inputs: &[],
        menu: None,
        field: None,
        overloads: EMPTY,
    },
    SignatureSpec {
        source: "looks_size",
        opcode: "size",
        inputs: &[],
        menu: None,
        field: None,
        overloads: EMPTY,
    },
    SignatureSpec {
        source: "looks_costumenumbername",
        opcode: "costume_number",
        inputs: &[],
        menu: None,
        field: Some("NUMBER_NAME"),
        overloads: COSTUME_NUMBER_NAME,
    },
    SignatureSpec {
        source: "looks_backdropnumbername",
        opcode: "backdrop_number",
        inputs: &[],
        menu: None,
        field: Some("NUMBER_NAME"),
        overloads: BACKDROP_NUMBER_NAME,
    },
    SignatureSpec {
        source: "sound_volume",
        opcode: "volume",
        inputs: &[],
        menu: None,
        field: None,
        overloads: EMPTY,
    },
    SignatureSpec {
        source: "sensing_distanceto",
        opcode: "distance_to",
        inputs: &["DISTANCETOMENU"],
        menu: Some("DISTANCETOMENU"),
        field: Some("DISTANCETOMENU"),
        overloads: DISTANCE_TO,
    },
    SignatureSpec {
        source: "sensing_touchingobject",
        opcode: "touching",
        inputs: &["TOUCHINGOBJECTMENU"],
        menu: Some("TOUCHINGOBJECTMENU"),
        field: Some("TOUCHINGOBJECTMENU"),
        overloads: TOUCHING,
    },
    SignatureSpec {
        source: "sensing_keypressed",
        opcode: "key_pressed",
        inputs: &["KEY_OPTION"],
        menu: Some("KEY_OPTION"),
        field: Some("KEY_OPTION"),
        overloads: EMPTY,
    },
    SignatureSpec {
        source: "sensing_mousedown",
        opcode: "mouse_down",
        inputs: &[],
        menu: None,
        field: None,
        overloads: EMPTY,
    },
    SignatureSpec {
        source: "sensing_mousex",
        opcode: "mouse_x",
        inputs: &[],
        menu: None,
        field: None,
        overloads: EMPTY,
    },
    SignatureSpec {
        source: "sensing_mousey",
        opcode: "mouse_y",
        inputs: &[],
        menu: None,
        field: None,
        overloads: EMPTY,
    },
    SignatureSpec {
        source: "sensing_loudness",
        opcode: "loudness",
        inputs: &[],
        menu: None,
        field: None,
        overloads: EMPTY,
    },
    SignatureSpec {
        source: "sensing_timer",
        opcode: "timer",
        inputs: &[],
        menu: None,
        field: None,
        overloads: EMPTY,
    },
    SignatureSpec {
        source: "sensing_current",
        opcode: "current_year",
        inputs: &[],
        menu: None,
        field: Some("CURRENTMENU"),
        overloads: CURRENT,
    },
    SignatureSpec {
        source: "sensing_dayssince2000",
        opcode: "days_since_2000",
        inputs: &[],
        menu: None,
        field: None,
        overloads: EMPTY,
    },
    SignatureSpec {
        source: "sensing_username",
        opcode: "username",
        inputs: &[],
        menu: None,
        field: None,
        overloads: EMPTY,
    },
    SignatureSpec {
        source: "sensing_touchingcolor",
        opcode: "touching_color",
        inputs: &["COLOR"],
        menu: None,
        field: None,
        overloads: EMPTY,
    },
    SignatureSpec {
        source: "sensing_coloristouchingcolor",
        opcode: "color_is_touching_color",
        inputs: &["COLOR", "COLOR2"],
        menu: None,
        field: None,
        overloads: EMPTY,
    },
    SignatureSpec {
        source: "sensing_answer",
        opcode: "answer",
        inputs: &[],
        menu: None,
        field: None,
        overloads: EMPTY,
    },
    SignatureSpec {
        source: "operator_random",
        opcode: "random",
        inputs: &["FROM", "TO"],
        menu: None,
        field: None,
        overloads: EMPTY,
    },
    SignatureSpec {
        source: "operator_length",
        opcode: "length",
        inputs: &["STRING"],
        menu: None,
        field: None,
        overloads: EMPTY,
    },
    SignatureSpec {
        source: "operator_round",
        opcode: "round",
        inputs: &["NUM"],
        menu: None,
        field: None,
        overloads: EMPTY,
    },
    SignatureSpec {
        source: "operator_mathop",
        opcode: "abs",
        inputs: &["NUM"],
        menu: None,
        field: Some("OPERATOR"),
        overloads: MATHOP,
    },
];

pub(crate) fn stmt_signature(opcode: &str) -> Option<Signature> {
    STMT_SIGNATURES
        .iter()
        .copied()
        .find(|spec| spec.source == opcode)
        .map(SignatureSpec::to_signature)
}

pub(crate) fn expr_signature(opcode: &str) -> Option<Signature> {
    EXPR_SIGNATURES
        .iter()
        .copied()
        .find(|spec| spec.source == opcode)
        .map(SignatureSpec::to_signature)
}

pub(crate) fn is_signature_name(name: &str) -> bool {
    STMT_SIGNATURES.iter().chain(EXPR_SIGNATURES).any(|spec| {
        spec.opcode == name || spec.overloads.iter().any(|(_, overload)| *overload == name)
    })
}
