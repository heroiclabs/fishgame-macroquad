var client;
var match_id;
var match;
var session;
var nakama_socket;
var received_buffer = [];
var events = [];
var authenticated = false;
var in_progress = false;
var error = ""
var records = ""

async function nakama_join_quick_match() {
    if (in_progress) {
        console.error("Operation in progress, impossible to make new request.");
        return;
    }

    in_progress = true;
    // Socket is open.
    console.log(nakama_socket);

    var response = await client.rpc(session, "rpc_macroquad_find_match", {
        "kind": "public",
        "engine": "macroquad",
    });
    match = await nakama_socket.joinMatch(response.payload.match_id);
    match_id = match.match_id;
    console.log("Joined match, id: %o", match_id);

    start_match();
    in_progress = false;
}

async function start_match() {
    if (match.presences != undefined) {
        match.presences.forEach((presence) => {
            if (presence.session_id !== match.self.session_id) {
                events.push({
                    event: 1,
                    user_id: presence.session_id,
                    username: presence.username,
                });

            }
        });
    }

    nakama_socket.onmatchdata = (result) => {
        var content = result.data;
        var buffer = new Uint8Array(Object.values(content));

        received_buffer.push({
            opcode: result.op_code,
            data: buffer,
            user_id: result.presence.session_id
        });    
    };

    nakama_socket.onmatchpresence = (matchpresence) => {
        console.info("Received match presence update:", matchpresence);

        if (matchpresence.leaves != undefined) {
            for (const i in matchpresence.leaves) {
                events.push({
                    event: 2,
                    user_id: matchpresence.leaves[i].session_id,
                });
            }
        }

        if (matchpresence.joins != undefined) {
            for (const i in matchpresence.joins) {
                if (matchpresence.joins[i].session_id !== match.self.session_id) {
                    events.push({
                        event: 1,
                        user_id: matchpresence.joins[i].session_id,
                        username: matchpresence.joins[i].username,
                    });
                }
            }
        }
    };
}

async function nakama_join_match(match_id) {
    in_progress = true;
    var match_id_string = consume_js_object(match_id);

    error = "";

    try {
        window.match = await nakama_socket.joinMatch(match_id_string);
        window.match_id = match.match_id;
        start_match();
        in_progress = false;
    } catch (error) {
        console.error(error);
        window.error = "Invalid match id";
        in_progress = false;
    }
}

function nakama_logout() {
    nakama_socket = undefined;
    session = undefined;
    authenticated = false;
}

function nakama_leave_match() {
    nakama_socket.leaveMatch(match_id);
    match = undefined;
    match_id = undefined;
}

function nakama_match_id() {
    if (match_id == undefined) {
        return -1;
    }

    return js_object(match_id);
}

function nakama_error() {
    if (error == "") {
        return -1;
    }

    return js_object(error);
}

function nakama_in_progress() {
    return in_progress;
}

function nakama_authenticated() {
    return authenticated;
}

async function nakama_create_private_match() {
    in_progress = true;

    error = "";

    window.match = await nakama_socket.createMatch();
    console.log("Created match:", window.match);
    
    window.match_id = window.match.match_id;

    start_match();

    in_progress = false;
}

async function nakama_add_matchmaker() {
    in_progress = true;

    error = "";

    // TODO: pass this from gui
    const minCount = 2;
    const maxCount = 4;
    const stringProperties = {
        engine: "macroquad_matchmaking"
    };
    const query = "+properties.engine:\"macroquad_matchmaking\"";
    var ticket = await nakama_socket.addMatchmaker(query, minCount, maxCount, stringProperties);

    nakama_socket.onmatchmakermatched = async (matched) => {
        console.info("Received MatchmakerMatched message: ", matched);
        match = await nakama_socket.joinMatch(match_id, matched.token);
        match_id = match.match_id;
        start_match();
        in_progress = false;
    };
}

function nakama_connected() {
    return match_id != undefined;
}

function nakama_self_id() {
    return js_object(match.self.session_id);
}

function nakama_try_recv() {
    if (received_buffer.length != 0) {
        return js_object(received_buffer.shift())
    }
    return -1;
}

function nakama_events() {
    if (events.length != 0) {
        return js_object(events.shift())
    }
    return -1;
}

function nakama_send(opcode, data) {
    if (match_id == undefined) {
        console.log("Not joined a match yet")
    }
    var id = match_id;
    var array = consume_js_object(data);
    var a = nakama_socket.sendMatchState(id, opcode, array);
}

function nakama_connect(key, server, port, protocol) {
    var key_string = consume_js_object(key);
    var server_string = consume_js_object(server);
    var protocol_string = consume_js_object(protocol);

    client = new nakamajs.Client(key_string, server_string, port, protocol_string);
    client.ssl = protocol_string == "https";
}

async function authenticate_or_register(email, password, register, username) {
    if (in_progress) {
        console.error("Operation in progress, impossible to make new request.");
        return;
    }

    in_progress = true;
    error = "";
    try {
        session = await client.authenticateEmail(email, password, register, username);
        console.info(session);
    } catch (error_message) {
        // TODO: figure why the real error message is not here
        console.log(error_message);
        in_progress = false;
        error = "Nakama request failed, probably invalid credentials";
        return;
    }

    const secure = client.ssl;
    const trace = false;
    nakama_socket = client.createSocket(secure, trace);
    nakama_socket.ondisconnect = (evt) => {
        console.info("Disconnected", evt);
    };

    await nakama_socket.connect(session);

    authenticated = true;

    in_progress = false;
    
}

async function nakama_authenticate(email, password) {
    var email_string = consume_js_object(email);
    var password_string = consume_js_object(password);

    await authenticate_or_register(email_string, password_string, false);
}

async function nakama_register(email, password, username) {
    var email_string = consume_js_object(email);
    var password_string = consume_js_object(password);
    var username_string = consume_js_object(username);

    await authenticate_or_register(email_string, password_string, true, username_string);
}

function nakama_username() {
    if (session == undefined) {
        return -1;
    }

    return js_object(session.username);
}

async function nakama_add_leaderboard_win() {
    var record = await client.writeLeaderboardRecord(session, "fish_game_macroquad_wins", {score: 1});
}

async function nakama_load_leaderboard_records() {
    in_progress = true;

    var records = [];
    var result = await client.listLeaderboardRecords(session, "fish_game_macroquad_wins");
    for (const i in result.records) {
        var record = result.records[i];
        records.push({ username: record.username, score: record.score });
    }

    if (result.next_cursor) {
        result = await client.listLeaderboardRecords(session, "fish_game_macroquad_wins", null, null, result.next_cursor);
        for (const i in result.records) {
            var record = result.records[i];
            records.push({ username: record.username, score: record.score });
        }
    }

    window.records = JSON.stringify(records);

    in_progress = false;
}

function nakama_leaderboard_records() {
    return js_object(window.records);
}

function register_plugin (importObject) {
    importObject.env.nakama_connect = nakama_connect;
    importObject.env.nakama_create_private_match = nakama_create_private_match;
    importObject.env.nakama_add_matchmaker = nakama_add_matchmaker;
    importObject.env.nakama_in_progress = nakama_in_progress;
    importObject.env.nakama_connected = nakama_connected;
    importObject.env.nakama_logout = nakama_logout;
    importObject.env.nakama_authenticated = nakama_authenticated;
    importObject.env.nakama_self_id = nakama_self_id;
    importObject.env.nakama_error = nakama_error;
    importObject.env.nakama_username = nakama_username;
    importObject.env.nakama_send = nakama_send;
    importObject.env.nakama_try_recv = nakama_try_recv;
    importObject.env.nakama_events = nakama_events;
    importObject.env.nakama_authenticate = nakama_authenticate;
    importObject.env.nakama_register = nakama_register
    importObject.env.nakama_join_quick_match = nakama_join_quick_match;
    importObject.env.nakama_join_match = nakama_join_match;
    importObject.env.nakama_leave_match = nakama_leave_match;
    importObject.env.nakama_match_id = nakama_match_id;
    importObject.env.nakama_add_leaderboard_win = nakama_add_leaderboard_win;
    importObject.env.nakama_load_leaderboard_records = nakama_load_leaderboard_records;
    importObject.env.nakama_leaderboard_records = nakama_leaderboard_records;
}


miniquad_add_plugin({ register_plugin, version: "0.1.1", name: "quad_nakama" });

