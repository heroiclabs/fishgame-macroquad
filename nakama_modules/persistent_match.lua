local nk = require("nakama")

local module = "match"
local params = {}
local public_match_id = nk.match_create(module, params)

local module = "match"
local params = {}
local dev_match_id = nk.match_create(module, params)

local function get_public_match_id()
  return nk.json_encode({["match_id"] = public_match_id})
end

local function get_dev_match_id()
  return nk.json_encode({["match_id"] = dev_match_id})
end

nk.register_rpc(get_public_match_id, "public_match_id")
nk.register_rpc(get_dev_match_id, "dev_match_id")
