import React, { useEffect, useRef } from "react";
import { finished } from "stream";

export interface EvalGraphProps {
  white: number[],
  black: number[],
  width: number,
  height: number,
  doLastBlack: boolean,
  finished: boolean,
}

export default function EvalGraph(props: EvalGraphProps) {
  const canvasRef = useRef<HTMLCanvasElement>(null);
  const divRef = useRef<HTMLDivElement>(null);

  function draw() {
    const canvas = canvasRef.current;
    if(canvas === null) return;
    const context = canvas.getContext("2d");
    if(context === null) return;

    // clear background
    context.fillStyle = "#383c4a";
    context.fillRect(0, 0, props.width, props.height);

    // number of turns to show
    const numTurns = Math.max(props.white.length, props.black.length);
    // max/min eval magnitude
    const evalMax = Math.max(Math.max(...props.white), Math.max(...props.black));
    const evalMin = Math.min(Math.min(...props.white), Math.min(...props.black));

    const leftMargin = Math.max(`${evalMax}`.length, `${evalMin}`.length) * 7 + 5;
    const rightMargin = 10;

    const topMargin = 10;
    const bottomMargin = 20;

    // draw axis
    context.strokeStyle = "#888";
    context.lineWidth = 1;
    context.beginPath();
    context.moveTo(leftMargin, topMargin);
    context.lineTo(leftMargin, props.height - bottomMargin);
    context.stroke();
    context.beginPath();
    context.moveTo(leftMargin, props.height - bottomMargin);
    context.lineTo(props.width - rightMargin, props.height - bottomMargin);
    context.stroke();

    const graphWidth = props.width - leftMargin - rightMargin;
    const graphHeight = props.height - topMargin - bottomMargin;
    const turnWidth = graphWidth / numTurns;

    // convert an eval (turn, eval) to graph (x, y)
    function toPos(turn: number, value: number): [number, number] {
      return [
        turn * turnWidth + leftMargin + 3, 
        props.height - bottomMargin - graphHeight * ((value - evalMin) / (evalMax - evalMin)) 
      ];
    }

    // draw turn numbers
    context.fillStyle = "white";
    context.font = "12px monospace";
    context.textAlign = "left";
    context.textBaseline = "top";
    
    const turnInc = Math.ceil(20.0 / turnWidth);
    for(let turn = 0; turn < numTurns; turn+= turnInc) {
      context.fillText(`${turn + 1}`, turn * turnWidth + leftMargin, props.height - bottomMargin + 5, turnWidth * turnInc);
    }

    // draw eval y = 0
    context.beginPath();
    context.lineWidth = 0.5;
    context.moveTo(leftMargin, toPos(0, 0)[1]);
    context.lineTo(props.width - rightMargin, toPos(0, 0)[1]);
    context.stroke();

    // draw eval y-axis numbers
    context.textBaseline = "middle";
    context.textAlign = "right";
    const evalInc = Math.ceil(20.0 * (evalMax - evalMin) / graphHeight);
    for(let y = 0; y < evalMax - evalMin; y+=evalInc) {
      const drawY0 = toPos(0, y)[1];
      const drawY1 = toPos(0, -y)[1];
      if(drawY0 >= topMargin) {
        context.fillText(`${y}`, leftMargin - 5, drawY0, leftMargin - 5);
      }
      if(drawY1 <= props.height - bottomMargin) {
        context.fillText(`${-y}`, leftMargin - 5, drawY1, leftMargin - 5);
      }
    }

    // draw data
    const entries: [number[], string][] = [[props.white, "white"], [props.black, "black"]];
    entries.forEach(([data, color]) => {
      const skipLast = (color == "black" && !props.doLastBlack) ? 1 : (props.finished ? 1 : 0);
      context.strokeStyle = color;
      context.lineWidth = color == "black" ? 1 : 0.5;
      context.fillStyle = color;
      // draw dots
      for(let turn = 0; turn < numTurns - skipLast; turn++) {
        const y = data[turn];
        context.beginPath();
        context.arc(toPos(turn, y)[0], toPos(turn, y)[1], 2, 0, 2 * Math.PI);
        context.fill();
      }
      // draw lines between points
      for(let turn = 0; turn < numTurns - 1 - skipLast; turn++) {
        const pos0 = toPos(turn, data[turn]);
        const pos1 = toPos(turn + 1, data[turn + 1]);
        context.beginPath();
        context.moveTo(pos0[0], pos0[1]);
        context.lineTo(pos1[0], pos1[1]);
        context.stroke();
      }
    });

  }

  useEffect(draw);

  return (
    <div ref={divRef} className="evalGraph">
      <canvas ref={canvasRef} width={`${props.width}px`} height={`${props.height}px`}></canvas>
    </div>
  )
}