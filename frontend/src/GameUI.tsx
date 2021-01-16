import React, { useState } from 'react';
import BoardFENWrapper from './BoardFENWrapper';
import EvalGraph from './EvalGraph';
import { Game, GamesState } from './Games';
import { ResizableBox } from 'react-resizable';

export interface ClientInfoProps {
  game: Game;
}

export function ClientInfo(props: ClientInfoProps) {
  const [collapsed, setCollapsed] = useState(true);

  return (
    <>
      <div 
          className="client-info-expand"
          onClick={() => {
            setCollapsed((collapsed) => !collapsed);
          }}
        >
          {collapsed ? "➤" : "▼"} Toggle Player Reported Info
        </div>
      <div className="flex gameInfo">
        {!collapsed &&
          <>
            <div className="gameInfoCont">
              {props.game.client_data.map((data: {[key: string]: string}, i) => {
                const body = []
                for(const [key, value] of Object.entries(data)) {
                  if(key === "eval") continue;
                  body.push(
                    <tr>
                      <td className="playerName">{key}</td>
                      <td>{value}</td>
                    </tr>
                  );
                }
                return (
                  <div className="gameInfoDiv roundedDiv scroll">
                    <table>
                      <thead>
                        <tr>
                          <td>{i == 0 ? "White:" : "Black:"}</td>
                        </tr>
                      </thead>
                      <tbody>
                        {body}
                      </tbody>
                    </table>
                  </div>
                );
              })}
            </div>
            <EvalGraph 
              width={300} 
              height={200} 
              white={props.game.client_data[0].eval.split(" ").map(s => parseFloat(s))}
              black={props.game.client_data[1].eval.split(" ").map(s => parseFloat(s))}
              doLastBlack={props.game.fen.split(" ")[1] === "w"}
              finished={props.game.finished}
            ></EvalGraph>
          </>
        }
      </div>
    </>
  );
}

export interface GameUIProps {
  state: GamesState;
  gameId: number;
  onSetKey: (key: string | null) => void;
  onMakeMove: (move: string) => void;
}

export default function GameUI(props: GameUIProps) {
  const game = props.state.games[props.gameId];

  const white = props.state.players[game.white_id];
    const black = props.state.players[game.black_id];
    const scoreStr = props.state.gameScoreStr(game.id);
    let status = "Not Started";
    if(game.finished) {
      status = `${scoreStr}: ${game.score == 1 ? "White Win" : game.score == -1 ? "Black Win" : "Tie"}`;
    } else if(props.state.gameIsActive(game.id)) {
      status = game.fen.split(" ")[1] === "w" ? "White To Move" : "Black To Move";
    }
    return (
      <div className="gameContainer" key={game.id} id={`game-${game.id}`}>
        <div className="gameTitleContainer flex">
          <div className="gameTitleLeft">
            <div>{white.name}</div>
            {white.apikey !== null &&
              <>
                <div className="apikey">API Key: {white.apikey}</div>
                { props.state.curPlayerId !== white.id ?
                  <span 
                    className="playAs"
                    onClick={() => {props.onSetKey(white.apikey)}}
                  >Play as this player</span>
                  :
                  <div className="apikey">
                    Current Player
                  </div>
                }
              </>
            }
          </div>
          <div className="gameTitleCenter">
            vs
          </div>
          <div className="gameTitleRight">
            {black.name}
            {black.apikey !== null &&
              <>
                <div className="apikey">API Key: {black.apikey}</div>
                {props.state.curPlayerId !== black.id ?
                  <span
                    className="playAs"
                    onClick={() => {props.onSetKey(black.apikey)}}
                  >Play as this player</span>
                  :
                  <div className="apikey">
                    Current Player
                  </div>
                }
              </>
            }
          </div>
        </div>
        <div className="flex">
          <div className="flexExpand gameFlexSide"/>
          <div className="game-width-limit">
            <div className="flex">
              <div className="flexExpand"></div>
              <div className="gameBody">
                <BoardFENWrapper 
                  fen={game.fen}
                  legal_moves={game.legal_moves}
                  size={400}
                  reversed={game.black_id === props.state.curPlayerId}
                  allow_moves={[game.white_id === props.state.curPlayerId, game.black_id === props.state.curPlayerId]}
                  onMove={props.onMakeMove}/>
              </div>
              <div className="gameMoves flex">
                <table className="gameMovesTable roundedDiv scroll">
                  <tbody>
                    {game.moves.filter((e, i) => i % 2 === 0).map((e, i) => (
                      <tr key={i}>
                        <td className="moveNum">{i + 1}.</td>
                        <td className="move">{game.moves[i * 2]}</td>
                        {game.moves.length > (i * 2 + 1) &&
                          <td className="move">{game.moves[i * 2 + 1]}</td>
                        }
                      </tr>
                    ))}
                  </tbody>
                </table>
                <div className="gameStatus roundedDiv">{status}</div>
              </div>
              <div className="flexExpand"></div>
            </div>
            <ClientInfo
              game={game}
            />
          </div>
          <div className="flexExpand gameFlexSide"/>
        </div>
      </div>
    );
}