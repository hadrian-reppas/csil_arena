package main

type Player struct {
	move uint32
}

func NewPlayer(board *[16][16]Cell) Player {
	return Player{0}
}

func (p *Player) Play(board *[16][16]Cell) Cell {
	p.move += 1
	return Cell(p.move % 8)
}
