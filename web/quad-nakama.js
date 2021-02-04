var match_id;
var match;
var nakama_socket;
var received_buffer = [];
var events = [];

async function init_nakama(key, server, port, protocol) {
    var client = new nakamajs.Client(key, server, port, protocol);
    client.ssl = protocol == "https";

    var email = "super@heroes.com";
    var password = "batsignal";
    const session = await client.authenticateEmail(email, password);
    console.info(session);

    const secure = protocol == "https";
    const trace = false;
    nakama_socket = client.createSocket(secure, trace);
    nakama_socket.ondisconnect = (evt) => {
        console.info("Disconnected", evt);
    };

    await nakama_socket.connect(session);
    // Socket is open.
    console.log(nakama_socket);

    nakama_socket.onchannelmessage = (message) => {
        console.info("Message received from channel", message.channel_id);
        console.info("Received message", message);
    };

    var response = await client.rpc(session, "rpc_macroquad_find_match", {
        "kind": "public",
        "engine": "macroquad",
    });
    console.log(response);
    match = await nakama_socket.joinMatch(response.payload.match_id);
    match_id = match.match_id;
    console.log("Joined match, id: %o", match_id);

    if (match.presences != undefined) {
        match.presences.forEach((presence) => {
            if (presence.session_id !== match.self.session_id) {
                events.push({
                    event: 1,
                    user_id: presence.session_id,
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
                    });
                }
            }
        }
    };
}

function nakama_is_connected() {
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

    init_nakama(key_string, server_string, port, protocol_string);
}

function register_plugin (importObject) {
    importObject.env.nakama_connect = nakama_connect;
    importObject.env.nakama_is_connected = nakama_is_connected;
    importObject.env.nakama_self_id = nakama_self_id;
    importObject.env.nakama_send = nakama_send;
    importObject.env.nakama_try_recv = nakama_try_recv;
    importObject.env.nakama_events = nakama_events;
}


miniquad_add_plugin({ register_plugin, version: "0.1.1", name: "quad_nakama" });

