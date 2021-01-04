import React, { useEffect, useState, Fragment } from 'react';
import BoardFENWrapper from './BoardFENWrapper';

export interface GameProps {
  clib: any,
  size: number,
  allow_moves: [boolean, boolean],
  reversed: boolean,
}

export default function Game(props: GameProps) {
  const [fen, setFen_internal] = useState("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1");
  const [legalMoves, setLegalMoves] = useState<string[]>([]);

  function setFen(fenNew: string) {
    setFen_internal(fenNew);
    const moves_str: string = props.clib.ccall("clib_board_legal_moves", "string", ["string"], [fenNew]);
    const moves = moves_str.split(",");
    setLegalMoves(moves);
  }

  function moveCallback(move: string) {
    const newFen: string = props.clib.ccall("clib_board_make_move", "string", ["string", "string"], [fen, move]);
    setFen(newFen);
  }

  useEffect(() => {setFen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1");}, []);

  return (
    <Fragment>
      <div>
        <BoardFENWrapper 
          fen={fen}
          legal_moves={legalMoves}
          size={props.size}
          reversed={props.reversed}
          allow_moves={props.allow_moves}
          onMove={moveCallback}
        />
      </div>
      <div>
        FEN: <code>{fen}</code>
      </div>
    </Fragment>
  );
}