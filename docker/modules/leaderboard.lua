local nk = require("nakama")

nk.run_once(function(context)
	nk.leaderboard_create("fish_game_macroquad_wins", false, "desc", "incr")
end)

