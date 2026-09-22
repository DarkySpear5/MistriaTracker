function mistria_tracker_companion_boot() {
    if (variable_global_exists("mistria_tracker_companion_runtime")
        && global.mistria_tracker_companion_runtime.registered) {
        return;
    }

    global.mistria_tracker_companion_runtime = {
        registered: true,
        sequence: 0,
        session_id: mistria_tracker_companion_session_id(),
        profile_id: undefined,
        pending_gift: undefined,
        awaiting_session_activation: true,
    };

    mmapi_mod_declare("mistria_tracker_companion", "0.1.6");
    mmapi_register(mistria_tracker_companion_tick);
    mmapi_filter("items.give", mistria_tracker_companion_items_give);
    mmapi_on("npc.gift_received", mistria_tracker_companion_gift_received);
    mmapi_on("museum.donate_item", mistria_tracker_companion_museum_donate_item);
    mmapi_on("game.room_changed", mistria_tracker_companion_room_changed);
    mmapi_on("game.title_entered", mistria_tracker_companion_title_entered);
}

function mistria_tracker_companion_session_id() {
    // `get_timer()` is a numeric GameMaker runner API. Pad and retain its last
    // twelve digits to form a valid UUID suffix without a hash or random API.
    var _clock = "000000000000" + string(get_timer());
    return "00000000-0000-7000-8000-"
        + string_copy(_clock, string_length(_clock) - 11, 12);
}

function mistria_tracker_companion_profile_id() {
    // `game.room_changed` also fires while the title screen is loading, before
    // the Game object has a save path. Stay completely inactive until a save
    // has been selected; this reads only the in-memory path once it exists.
    var _basename = mistria_tracker_companion_save_file();
    if (_basename == undefined) return undefined;
    var _prefix = "game-";
    var _start = string_pos(_prefix, _basename);
    if (_start != 1) return undefined;

    var _profile_start = string_length(_prefix) + 1;
    var _profile_and_suffix = string_delete(_basename, 1, _profile_start - 1);
    var _end = string_pos("-", _profile_and_suffix);
    if (_end <= 1) return undefined;

    var _profile_id = string_copy(_profile_and_suffix, 1, _end - 1);
    for (var _index = 1; _index <= string_length(_profile_id); _index += 1) {
        if (string_pos(string_char_at(_profile_id, _index), "0123456789") == 0) return undefined;
    }
    return _profile_id;
}

function mistria_tracker_companion_save_file() {
    var _path = undefined;
    try {
        _path = Game.last_serde_path;
    } catch (_error) {
        return undefined;
    }
    if (_path == undefined) return undefined;
    return filename_name(_path);
}

function mistria_tracker_companion_emit(_type, _payload, _save_file) {
    var _profile_id = undefined;
    if (_type == "profile_deactivated") {
        _profile_id = global.mistria_tracker_companion_runtime.profile_id;
    } else {
        _profile_id = mistria_tracker_companion_profile_id();
    }
    if (_profile_id == undefined) return;

    global.mistria_tracker_companion_runtime.profile_id = _profile_id;
    global.mistria_tracker_companion_runtime.sequence += 1;
    var _event = {
        schema_version: 1,
        companion_version: "0.1.6",
        game_version: "1.0.5",
        profile_id: _profile_id,
        session_id: global.mistria_tracker_companion_runtime.session_id,
        sequence: global.mistria_tracker_companion_runtime.sequence,
        type: _type,
    };
    if (_payload != undefined) _event.payload = _payload;
    if (_save_file != undefined) _event.save_file = _save_file;

    mmapi_log_info("mistria_tracker_companion", "MISTRIA_TRACKER_EVENT|" + json_stringify(_event));
    mmapi_log_flush("mistria_tracker_companion");
}

function mistria_tracker_companion_items_give(_value, _ctx) {
    try {
        if (_value == undefined) return undefined;
        if (mistria_tracker_companion_profile_id() == undefined) return undefined;
        mistria_tracker_companion_emit("item_obtained", {
            item_id: item_id_to_string(_value.item_id),
            count: _value.count,
        }, undefined);
    } catch (_error) { }
    return undefined;
}

function mistria_tracker_companion_gift_received(_ctx) {
    if (_ctx == undefined || mistria_tracker_companion_profile_id() == undefined) return;
    try {
        global.mistria_tracker_companion_runtime.pending_gift = {
            npc_id: item_id_to_string(_ctx.npc.npc_id),
            item_id: item_id_to_string(_ctx.item),
        };
    } catch (_error) { }
}

function mistria_tracker_companion_tick() {
    if (global.mistria_tracker_companion_runtime.awaiting_session_activation) {
        var _save_file = mistria_tracker_companion_save_file();
        var _profile_id = mistria_tracker_companion_profile_id();
        if (_save_file != undefined && _profile_id != undefined) {
            global.mistria_tracker_companion_runtime.session_id = mistria_tracker_companion_session_id();
            global.mistria_tracker_companion_runtime.sequence = 0;
            global.mistria_tracker_companion_runtime.awaiting_session_activation = false;
            mistria_tracker_companion_emit("profile_activated", undefined, _save_file);
        }
    }
    if (global.mistria_tracker_companion_runtime.pending_gift == undefined) return;
    mistria_tracker_companion_emit_pending_gift();
}

function mistria_tracker_companion_title_entered(_ctx) {
    mistria_tracker_companion_emit("profile_deactivated", undefined, undefined);
    global.mistria_tracker_companion_runtime.awaiting_session_activation = true;
    global.mistria_tracker_companion_runtime.profile_id = undefined;
    global.mistria_tracker_companion_runtime.pending_gift = undefined;
}

function mistria_tracker_companion_emit_pending_gift() {
    if (mistria_tracker_companion_profile_id() == undefined) return;
    var _pending = global.mistria_tracker_companion_runtime.pending_gift;
    global.mistria_tracker_companion_runtime.pending_gift = undefined;
    if (_pending == undefined || array_length(GAME_STATS.gifts_given) <= 0) return;

    var _recorded = GAME_STATS.gifts_given[array_length(GAME_STATS.gifts_given) - 1];
    if (_recorded.npc_id != _pending.npc_id || _recorded.item_id != _pending.item_id) return;
    mistria_tracker_companion_emit("gift_given", {
        npc_id: _recorded.npc_id,
        item_id: _recorded.item_id,
        reaction: _recorded.reaction,
    }, undefined);
}

function mistria_tracker_companion_museum_donate_item(_ctx) {
    if (_ctx == undefined || mistria_tracker_companion_profile_id() == undefined) return;
    mistria_tracker_companion_emit("museum_donated", {
        item_id: item_id_to_string(_ctx.item_id),
    }, undefined);
}

function mistria_tracker_companion_room_changed(_ctx) {
    var _profile_id = mistria_tracker_companion_profile_id();
    if (_profile_id == undefined) return;
    var _save_file = mistria_tracker_companion_save_file();
    if (_save_file == undefined) return;
    mistria_tracker_companion_emit("profile_activated", undefined, _save_file);
}

mistria_tracker_companion_boot();
