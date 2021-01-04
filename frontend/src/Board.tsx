import React, { useEffect, useRef, useState } from 'react';
import { ParsedMove } from './BoardFENWrapper';

export interface BoardTheme {
  square1color: string,
  square2color: string,
  cur_square_color: string,
  moves_hint_color: string,
  icon_name: string,
  rank_file_text_color: string,
  outer_background: string,
}

export interface BoardProps {
  size: number,
  reversed: boolean,
  theme: BoardTheme,
  pieces: string[][],
  player: number,
  moves: ParsedMove[],
  allow_moves: [boolean, boolean],
  moveCallback: (move: ParsedMove) => void,
}

const piece_icons: {[theme: string]: {[piece: string]: HTMLImageElement}} = {
  "default": {
    "p": new Image(),
    "r": new Image(),
    "n": new Image(),
    "b": new Image(),
    "q": new Image(),
    "k": new Image(),
    "P": new Image(),
    "R": new Image(),
    "N": new Image(),
    "B": new Image(),
    "Q": new Image(),
    "K": new Image(),
  }
};

let piece_icons_num_total = 0;
let piece_icons_loaded = 0;

function piece_icons_onload_callback() {
  piece_icons_loaded++;
}

for(const theme in piece_icons) {
  for(const piece in piece_icons[theme]) {
    piece_icons[theme][piece].src = `/icons/${theme}/${piece}.svg`;
    piece_icons[theme][piece].onload = piece_icons_onload_callback;
    piece_icons_num_total++;
  }
}

// given a pice char, get its player index
function pieceToPlayer(piece: string): number {
  return piece[0] === piece[0].toUpperCase() ? 0 : 1;
}

// given a promotion piece and player, get the right piece string
function promotePieceToPiece(piece: string, player: number): string {
  if(player === 0) {
    return piece.toUpperCase();
  } else {
    return piece.toLowerCase();
  }
}

interface PromoteState {
  dst: [number, number],
  promote_display_locs: [number, number][],
  promote_piece: string[],
}

export default function Board(props: BoardProps) {
  const canvasRef = useRef<HTMLCanvasElement>(null);
  const [curSquare, setCurSquare] = useState<[number, number] | null>(null);
  const [curPromote, setCurPromote] = useState<PromoteState | null>(null);

  // map a board x, y to a display x, y
  function posToDisplayXY(x: number, y: number): [number, number] {
    if(props.reversed) {
      return [x, y];
    } else {
      return [x, 7-y];
    }
  }

  function filterMoves(moves: ParsedMove[], src: [number, number]): ParsedMove[] {
    return moves.filter(move => move[0][0] === src[0] && move[0][1] === src[1]);
  }

  // size of inner board (we have to display rank + file indicators outside)
  const x_off = 20;
  const y_off = x_off;
  const inner_size = props.size - x_off * 2;
  // size of one square on board
  const square_size = inner_size / 8;

  function draw() {
    const canvas = canvasRef.current;
    if(canvas === null) return;
    const context = canvas.getContext("2d");
    if(context === null) return;

    if(piece_icons_loaded < piece_icons_num_total) {
      context.fillStyle = "white";
      context.fillRect(0, 0, props.size, props.size);
      context.fillStyle = "black";
      context.textAlign = "center";
      context.textBaseline = "middle";
      context.font = "14px monospace";
      context.fillText("Loading Icons ...", props.size / 2, props.size / 2);
      setTimeout(draw, 500);
      return;
    }

    const moves = curSquare !== null ? filterMoves(props.moves, curSquare) : [];

    // clear canvas
    context.fillStyle = props.theme.outer_background;
    context.fillRect(0, 0, props.size, props.size);
    // draw board
    for(let x = 0; x < 8; x++) {
      for(let y = 0; y < 8; y++) {
        const [dis_x, dis_y] = posToDisplayXY(x, y);
        let back_color = dis_x % 2 === dis_y % 2 ? props.theme.square1color : props.theme.square2color;
        // color curSquare
        if(curSquare != null && curSquare[0] === x && curSquare[1] === y) {
          back_color = props.theme.cur_square_color;
        }
        context.fillStyle = back_color;
        context.fillRect(x_off + dis_x * square_size, y_off + dis_y * square_size, square_size, square_size);
        // draw piece
        const piece = props.pieces[y][x];
        if(piece !== "") {
          context.drawImage(piece_icons[props.theme.icon_name][piece], x_off + dis_x * square_size, y_off + dis_y * square_size, square_size, square_size);
        }
      }
    }
    // draw legal move indicators
    let prev_drawn: [number, number][] = [];
    moves.forEach(move => {
      const [x, y] = move[1];
      if(prev_drawn.filter(move => x === move[0] && y === move[1]).length >= 1) {
        return;
      }
      const [dis_x, dis_y] = posToDisplayXY(x, y);
      context.strokeStyle = props.theme.moves_hint_color;
      context.lineWidth = props.pieces[y][x] === "" ? square_size / 5 : 5;
      const radius = props.pieces[y][x] === "" ? square_size / 10 : square_size / 2.5;
      context.beginPath();
      context.arc(x_off + (dis_x + 0.5) * square_size, y_off + (dis_y + 0.5) * square_size, radius, 0, 2*Math.PI);
      context.stroke();
      prev_drawn.push([x, y]);
    });
    // draw promotion options
    if(curPromote != null) {
      curPromote.promote_display_locs.forEach((dst, index) => {
        const [dis_x, dis_y] = posToDisplayXY(dst[0], dst[1]);
        context.fillStyle = "white";
        context.fillRect(x_off + dis_x * square_size, y_off + dis_y * square_size, square_size, square_size);
        context.strokeStyle = "black";
        context.lineWidth = 1;
        context.strokeRect(x_off + dis_x * square_size, y_off + dis_y * square_size, square_size, square_size);
        const piece = promotePieceToPiece(curPromote.promote_piece[index], props.player);
        context.drawImage(piece_icons[props.theme.icon_name][piece], x_off + dis_x * square_size, y_off + dis_y * square_size, square_size, square_size);
      });
    }
    // draw rank and file indicators
    context.font = "14px monospace";
    context.fillStyle = props.theme.rank_file_text_color;
    context.textAlign = "center";
    context.textBaseline = "middle";
    for(let x = 0; x < 8; x++) {
      const dis_x = posToDisplayXY(x, 0)[0];
      context.fillText(`${"abcdefgh"[x]}`, x_off + (dis_x + 0.5) * square_size, y_off / 2);
      context.fillText(`${"abcdefgh"[x]}`, x_off + (dis_x + 0.5) * square_size, y_off * 1.5 + 8 * square_size);
    }
    for(let y = 0; y < 8; y++) {
      const dis_y = posToDisplayXY(0, y)[1];
      context.fillText(`${"12345678"[y]}`, x_off / 2, y_off + (dis_y + 0.5) * square_size);
      context.fillText(`${"12345678"[y]}`, x_off * 1.5 + 8 * square_size, y_off + (dis_y + 0.5) * square_size);
    }
  }

  function handleClick(e: React.MouseEvent<HTMLElement>) {
    if(!props.allow_moves[props.player]) {
      return;
    }
    if(canvasRef.current === null) {
      return;
    }
    const canvas_x = e.pageX - canvasRef.current.getBoundingClientRect().left - x_off;
    const canvas_y = e.pageY - canvasRef.current.getBoundingClientRect().top - y_off;
    const dis_square_x = Math.floor(canvas_x / square_size);
    const dis_square_y = Math.floor(canvas_y / square_size);
    if(dis_square_x < 0 || dis_square_x >= 8 || dis_square_y < 0 || dis_square_y >= 8) {
      return;
    }

    const [x, y] = posToDisplayXY(dis_square_x, dis_square_y);
    const piece = props.pieces[y][x];

    // handle promotion
    if(curPromote !== null) {
      for(let i = 0; i < curPromote.promote_display_locs.length; i++) {
        const pos = curPromote.promote_display_locs[i];
        if(x === pos[0] && y === pos[1]) {
          const promotePiece = curPromote.promote_piece[i];
          props.moveCallback([curSquare!, curPromote.dst, promotePiece]);
          setCurPromote(null);
          setCurSquare(null);
          return;
        }
      }
    }
    // select / deselect our pieces
    if(piece !== "" && pieceToPlayer(piece) === props.player) {
      // deselect if we clicked the active square
      if(curSquare !== null && curSquare[0] === x && curSquare[1] === y) {
        setCurSquare(null);
        setCurPromote(null);
      } else {
        setCurSquare([x, y]);
        setCurPromote(null);
      }
    } else if(curSquare !== null) {
      // check if this square is a legal move
      const moves = filterMoves(props.moves, curSquare);
      const movesToCurSquare = moves.filter(move => move[1][0] === x && move[1][1] === y);
      // if only one move to this square exists, make it
      if(movesToCurSquare.length === 1) {
        props.moveCallback(movesToCurSquare[0]);
        setCurSquare(null);
      }
      // otherwise, display promotion options
      else if(movesToCurSquare.length > 1) {
        setCurPromote({
          dst: [x, y],
          promote_piece: movesToCurSquare.map(move => move[2]!),
          promote_display_locs: movesToCurSquare.map((move, index) => [x, y >= 4 ? y - index : y + index])
        });
      } else {
        setCurSquare(null);
        setCurPromote(null);
      }
    }
  }

  useEffect(draw);

  return (
    <canvas ref={canvasRef} width={`${props.size}px`} height={`${props.size}px`} onClick={handleClick}></canvas>
  );
}

