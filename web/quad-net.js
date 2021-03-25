function on_init() {
}

register_plugin = function (importObject) {
    importObject.env.ws_connect = ws_connect;
    importObject.env.ws_is_connected = ws_is_connected;
    importObject.env.ws_send = ws_send;
    importObject.env.ws_try_recv = ws_try_recv;

    importObject.env.http_make_request = http_make_request;
    importObject.env.http_try_recv = http_try_recv;
}

miniquad_add_plugin({ register_plugin, on_init, version: "0.1.1", name: "quad_net" });

var quad_socket;
var connected = 0;
var received_buffer = [];

function ws_is_connected() {
    return connected;
}

function ws_connect(addr) {
    quad_socket = new WebSocket(consume_js_object(addr));
    quad_socket.binaryType = 'arraybuffer';
    quad_socket.onopen = function() {
        connected = 1;
    };

    quad_socket.onmessage = function(msg) {
        if (typeof msg.data == "string") {
            received_buffer.push({
                "text": 1,
                "data": msg.data
            });
        } else {
            var buffer = new Uint8Array(msg.data);
            received_buffer.push({
                "text": 0,
                "data": buffer
            });
        }

    }
};

function ws_send(data) {
    var array = consume_js_object(data);
    // here should be a nice typecheck on array.is_string or whatever
    if (array.buffer != undefined) {
        quad_socket.send(array.buffer);
    } else {
        quad_socket.send(array);
    }
};

function ws_try_recv() {
    if (received_buffer.length != 0) {
        return js_object(received_buffer.shift())
    }
    return -1;
}


var uid = 0;
var ongoing_requests = {};

function http_try_recv(cid) {
    if (ongoing_requests[cid] != undefined && ongoing_requests[cid] != null) {
        var data = ongoing_requests[cid];
        ongoing_requests[cid] = null;
        return js_object(data);
    }
    return -1;
}

function http_make_request(scheme, url, body, headers) {
    var cid = uid;

    uid += 1;

    var scheme_string;
    if (scheme == 0) {
        scheme_string = 'POST';
    }
    if (scheme == 1) {
        scheme_string = 'PUT';
    }
    if (scheme == 2) {
        scheme_string = 'GET';
    }
    if (scheme == 3) {
        scheme_string = 'DELETE';
    }
    var url_string = consume_js_object(url);
    var body_string = consume_js_object(body);
    var headers_obj = consume_js_object(headers);
    var xhr = new XMLHttpRequest();
    xhr.open(scheme_string, url_string, true);
    xhr.responseType = 'arraybuffer';
    for (const header in headers_obj) {
        xhr.setRequestHeader(header, headers_obj[header]);
    }
    xhr.onload = function (e) {
        if (this.status == 200) {
            var uInt8Array = new Uint8Array(this.response);
            
            ongoing_requests[cid] = uInt8Array;
        }
    }
    xhr.onerror = function (e) {
        // todo: let rust know and put Error to ongoing requests
        console.error("Failed to make a request");
        console.error(e);
    };

    xhr.send(body_string);

    return cid;
}
