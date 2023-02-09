import WebSocket from "ws";
import dotenv from "dotenv";
import jwtDecode, { JwtPayload } from "jwt-decode";
import { Algorithm } from "./algorithm";

// Load the .dotenv configuration
dotenv.config();

// Parse the JWT and player ID
const jwt = process.env.JWT ?? "";
const playerId = jwtDecode<JwtPayload>(jwt)?.sub ?? "";

// Connect to the WebSocket server
const ws = new WebSocket("ws://localhost:53700/api/v1/play", [
  "game-server",
  jwt,
]);

// Start the algorithm
new Algorithm(playerId, ws);
