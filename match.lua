local nk = require("nakama")

local M = {}

function M.match_init(context, setupstate)
   local gamestate = {
      players = {}
   }
   local tickrate = 30
   local label = {
      kind = setupstate.kind;
      engine = setupstate.engine;
   }
   return gamestate, tickrate, nk.json_encode(label)
end

function M.match_join_attempt(context, dispatcher, tick, state, presence, metadata)
   local acceptuser = true
   return state, acceptuser
end

function M.match_join(context, dispatcher, tick, state, presences)
   for _, presence in ipairs(presences) do
      state.players[presence.session_id] = {
         presence = presence;
         tick = tick;
      }
   end
   return state
end

function M.match_leave(context, dispatcher, tick, state, presences)
   for _, presence in ipairs(presences) do
      state.players[presence.session_id] = nil
   end
   return state
end

function M.match_loop(context, dispatcher, tick, state, messages)
   for _, m in ipairs(messages) do
      dispatcher.broadcast_message(m.op_code, m.data, nil, m.sender)
      state.players[m.sender.session_id].tick = tick
   end

   for session_id, player in pairs(state.players) do
      if tick - player.tick > 200 then
         nk.logger_info(string.format("kicking %q", session_id))
         dispatcher.match_kick({ player.presence })
      end
   end
   return state
end

function M.match_terminate(context, dispatcher, tick, state, grace_seconds)
   return state
end

return M

