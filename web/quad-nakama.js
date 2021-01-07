var match_id;
var socket;
var received_buffer = [];

async function init_nakama() {
    var client = new nakamajs.Client("defaultkey", "173.0.157.169", 7350);
    client.ssl = false;

    console.log(client);

    var email = "super@heroes.com";
    var password = "batsignal";
    const session = await client.authenticateEmail(email, password);
    console.info(session);

    const secure = false; // Enable if server is run with an SSL certificate
    const trace = false;
    socket = client.createSocket(secure, trace);
    socket.ondisconnect = (evt) => {
        console.info("Disconnected", evt);
    };

    await socket.connect(session);
    // Socket is open.
    console.log(socket);

    socket.onchannelmessage = (message) => {
        console.info("Message received from channel", message.channel_id);
        console.info("Received message", message);
    };

    const query = "*";
    const minCount = 2;
    const maxCount = 2;

    var ticket = await socket.addMatchmaker(query, minCount, maxCount);

    socket.onmatchmakermatched = (matched) => {
        console.info("Received MatchmakerMatched message: ", matched);
        console.info("Matched opponents: ", matched.users);

        const message = {
            match_join: {
                token: matched.token
            }
        };
        socket.send(message).await;
    };

    socket.onmatchdata = (result) => {
        var content = result.data;
        switch (result.op_code) {
        case 101:
            console.log("A custom opcode.");
            break;
        default:
            var buffer = new Uint8Array(Object.values(content));
            received_buffer.push(buffer);
            break;
        }
    };
    socket.onmatchpresence = (matchpresence) => {
        console.info("Received match presence update:", matchpresence);
        match_id = matchpresence.match_id;
    };
}

function nakama_is_connected() {
    return match_id != undefined;
}

function nakama_try_recv() {
    if (received_buffer.length != 0) {
        return js_object(received_buffer.shift())
    }
    return -1;
}

function nakama_send(opcode, data) {
    if (match_id == undefined) {
        console.log("Not joined a match yet")
    }
    var id = match_id;
    var array = consume_js_object(data);
    socket.sendMatchState(id, opcode, array);
}

function register_plugin (importObject) {
    importObject.env.nakama_is_connected = nakama_is_connected;
    importObject.env.nakama_send = nakama_send;
    importObject.env.nakama_try_recv = nakama_try_recv;
}

miniquad_add_plugin({ register_plugin, version: "0.1.0", name: "quad_nakama" });

init_nakama();
