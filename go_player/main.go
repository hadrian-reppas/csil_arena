package main

import (
	"csil.com/internal/csil/arena/api"
	"go.bytecodealliance.org/cm"
)

func ParseBoard(bytes []uint8) [16][16]Cell {
	if len(bytes) != 256 {
		panic("input bytes must have length 256")
	}

	var board [16][16]Cell
	for i := range 256 {
		if bytes[i] > 9 {
			panic("input bytes must be in the range [0, 9]")
		}
		r := i / 16
		c := i % 16
		board[r][c] = Cell(bytes[i])
	}

	return board
}

var player *Player
var handle api.Player

func PlayerConstructor(bytes cm.List[uint8]) api.Player {
	if player != nil {
		panic("player must only be constructed once")
	}

	board := ParseBoard(bytes.Slice())

	instance := NewPlayer(&board)
	player = &instance

	// Always use the same handle
	handle = api.PlayerResourceNew(cm.Rep(1))
	return handle
}

func PlayerDestructor(p cm.Rep) {}

func PlayerPlay(p cm.Rep, bytes cm.List[uint8]) int64 {
	if player == nil {
		panic("player must be initialized")
	}

	var board = ParseBoard(bytes.Slice())

	move := player.Play(&board)
	return int64(move)
}

func init() {
	api.Exports.Player.Constructor = PlayerConstructor
	api.Exports.Player.Play = PlayerPlay
	api.Exports.Player.Destructor = PlayerDestructor
}

// Required for the wasi target even though it isn't used
func main() {}
