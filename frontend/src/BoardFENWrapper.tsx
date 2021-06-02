import React, { useEffect } from 'react';
import Board from './Board';

// convert a fen string to an 8x8 array of piece chars and a player to move
function parseFen(fen: string): [ string[][], number] {
  const res: string[][] = [];
  for(let i = 0; i < 8; i++) {
    res.push([]);
    for(let j = 0; j < 8; j++) {
      res[i].push('');
    }
  }

  // go through fen string and extract piece locations
  let fen_sections = fen.split(/[ ,]+/);
  let rows = fen_sections[0].split("/");
  for(let y = rows.length - 1; y >= 0; y--) {
    let row = rows[rows.length - y - 1];
    let x = 0;
    for(let i = 0; i < row.length; i++) {
      if(['1', '2', '3', '4', '5', '6', '7', '8'].includes(row[i])) {
        x += Number(row[i]);
      } else {
        res[y][x] = row[i];
        x++;
      }
    }
  }

  const player = fen_sections[1] === 'w' ? 0 : 1;

  return [res, player];
}

export type ParsedMove = [[number, number], [number, number], string | null];

function parseMoveStr(move: string): ParsedMove {
  const src_x = "abcdefgh".indexOf(move[0]);
  const src_y = "12345678".indexOf(move[1]);
  const dst_x = "abcdefgh".indexOf(move[2]);
  const dst_y = "12345678".indexOf(move[3]);
  const promote = move.length > 4 ? move[4] : null;
  return [[src_x, src_y], [dst_x, dst_y], promote];
}

function parsedMoveToMoveStr(move: ParsedMove): string {
  let res = `${"abcdefgh"[move[0][0]]}${"12345678"[move[0][1]]}${"abcdefgh"[move[1][0]]}${"12345678"[move[1][1]]}`;
  if(move[2] !== null) {
    res += move[2];
  }
  return res;
}

export interface BoardFENWrapperProps {
  fen: string,
  legal_moves: string[],
  size: number,
  reversed: boolean,
  allow_moves: [boolean, boolean],
  onMove: (move: string) => void,
  last_move: string | undefined,
}

const defaultTheme = {
  square1color: "#ffce9e",
  square2color: "#d18b47",
  cur_square_color: "rgba(255, 255, 0, 0.5)",
  moves_hint_color: "rgba(80, 80, 80, 0.3)",
  icon_name: "default",
  rank_file_text_color: "white",
  outer_background: "#2a2a2e",
};


export default function BoardFENWrapper(props: BoardFENWrapperProps) {
  const [pieces, player] = parseFen(props.fen);
  const moves = props.legal_moves.map(parseMoveStr);
  const last_move = props.last_move === undefined ? undefined : parseMoveStr(props.last_move);

  return (
    <Board 
      size={props.size} 
      reversed={props.reversed} 
      theme={defaultTheme} 
      pieces={pieces} 
      player={player} 
      moves={moves}
      allow_moves={props.allow_moves}
      moveCallback={(move) => { props.onMove(parsedMoveToMoveStr(move)); }}
      last_move={last_move} 
    />
  );
}
