import React from 'react';
import { GamesState } from "./Games";

export interface PlayersTableProps {
  state: GamesState;
}

export default function PlayersTable(props: PlayersTableProps) {
  const playersElem = [];
  for(const [id, player] of Object.entries(props.state.players)) {
    playersElem.push(
      <tr key={id}>
        <td className="playerName">{player.name}</td>
        <td>{`${player.wins}-${player.losses}-${player.ties}`}</td>
      </tr>
    );
  }

  const players = (
    <table className="playersTable">
      <thead>
        <tr>
          <th>Name</th>
          <th>Score</th>
        </tr>
      </thead>
      <tbody>
        {playersElem}
      </tbody>
    </table>
  );

  const gamesSidebarElems = [];
  for(const [id, game] of Object.entries(props.state.games)) {
    const white = props.state.players[game.white_id];
    const black = props.state.players[game.black_id];
    const statusMsg = game.finished ? props.state.gameScoreStr(game.id) : props.state.gameIsActive(game.id) ? "In Progress" : "No Started";
    gamesSidebarElems.push(
      <tr key={id}>
        <td className="playerName">{white.name} vs {black.name}</td>
        <td>{statusMsg}</td>
      </tr>
    );
  }

  const gamesSidebar = (
    <table className="playersTable">
      <thead>
        <tr>
          <th>Match</th>
          <th>Outcome</th>
        </tr>
      </thead>
      <tbody>
        {gamesSidebarElems}
      </tbody>
    </table>
  );

  return (
    <>
      <div className="playersTitle">Players</div>
      {players}
      <div className="playersTitle">Games</div>
      {gamesSidebar}

    </>
  );
}