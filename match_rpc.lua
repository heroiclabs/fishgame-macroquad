local nk = require("nakama")

local function rpc_macroquad_find_match(context, payload)
   local params = nk.json_decode(payload)
   local limit = 1
   local authoritative = true

   local min_size = 0
   local max_size = 4

   local query = string.format("+label.kind:%s +label.engine:%s", params.kind, params.engine)
   local matches = nk.match_list(limit, authoritative, label, min_size, max_size, query)

   local res

   if #matches == 0 then
      local module = "match"
      
      res = nk.match_create(module, params)
   else
      res = matches[1].match_id
   end

   return nk.json_encode({["match_id"] = res})
end

nk.register_rpc(rpc_macroquad_find_match, "rpc_macroquad_find_match")
