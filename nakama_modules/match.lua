local M = {}

function M.match_init(context, setupstate)
  local gamestate = {}
  local tickrate = 10
  local label = ""
  return gamestate, tickrate, label
end

function M.match_join_attempt(context, dispatcher, tick, state, presence, metadata)
  local acceptuser = true
  return state, acceptuser
end

function M.match_join(context, dispatcher, tick, state, presences)
  return state
end

function M.match_leave(context, dispatcher, tick, state, presences)
  return state
end

function M.match_loop(context, dispatcher, tick, state, messages)
  for _, m in ipairs(messages) do
      dispatcher.broadcast_message(m.op_code, m.data, nil, m.sender)
  end
  return state
end

function M.match_terminate(context, dispatcher, tick, state, grace_seconds)
  return state
end

return M

