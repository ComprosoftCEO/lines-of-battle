//DisplayLogic.JS
//Jonathan Keesling

//This script functions as the logic for the AreaDisplay

//global var to track current players
var globalPlayers = null; 
var globalPlayerOrder = null;
var globalPlayersKilled = []; 

//create buttons that allow user to select canvas size and connect to server. returns ref
function readyGameButtons()
{
    //create start button
    let startButton = document.createElement("button");
    startButton.innerHTML = "Start Game";
    startButton.id = "startGameButton"; 

    //get div that holds server address box and text
    var body = document.getElementsByTagName("body")[0];
    var mid = body.getElementsByClassName("middle_row")[0];
    var InfoInput = mid.getElementsByClassName("serverInfoInput")[0];

    //insert start button
    InfoInput.appendChild(startButton);

    //return button ref (mostly for testing rn)
    startButton = document.getElementById("startGameButton");
    return startButton; 
}

//create button for refreshing page after game has ended
function createRefreshButton()
{
    //create start button
    let refreshButton = document.createElement("button");
    refreshButton.innerHTML = "Refresh Arena View";
    refreshButton.id = "refresh-arena"; 

    //get div that holds right column
    var body = document.getElementsByTagName("body")[0];
    var mid = body.getElementsByClassName("middle_row")[0];
    var turnInfo = mid.getElementsByClassName("turnInfo")[0];

    //insert start button
    turnInfo.appendChild(refreshButton);

    //return button ref (mostly for testing rn)
    refreshButton = document.getElementById("refresh-arena");
    return refreshButton; 
}

//create canvas given width and height. Returns reference to canvas. 
function readyCanvas(givenWidth,givenheight)
{
    try
    {
        //generate canvas of given size
        var newCanvas = document.createElement('canvas');

        newCanvas.id = "battlefieldCanvas";
        newCanvas.width = givenWidth;
        newCanvas.height = givenheight;
        newCanvas.style.border = "2px solid";

        //append canvas to HML
        var body = document.getElementsByTagName("body")[0];
        var mid = body.getElementsByClassName("middle_row")[0];
        var middleMiddle_column = mid.getElementsByClassName("MM_column")[0];
        middleMiddle_column.appendChild(newCanvas);

        //return reference to canvas
        battlefieldCanvas = document.getElementById("battlefieldCanvas");
        return battlefieldCanvas; 
    }
    catch(error)
    {
        console.error(error);
        alert("Error readying canvas")

        //return null, no canvas made
        return null
    }
    
}

//make connection to gameserver and return websoceket ref
function connectToGameserver(addrAndPort,config)
{
    try
    {
        //form connection string
        conString ="ws://" + addrAndPort.trim() + config.viewer_path ; 

        //attempt connection
        const newSocket = new WebSocket(conString, ["game-server", config.jwt]);
        newSocket.onerror = (event) =>
        {
            //make sure our connection does not result in error
            console.log(event);
            alert("Error connecting to server");
            return null
        }
        return newSocket
    }
    catch(error)
    {
        console.error(error);
        alert("Error connection to server")

        //return null, no configs read
        return null
    }
}

//updates to page text to preform when server is waiting on players
function updatePage_onWaitingOnPlayers(msgData)
{
    //update server status
    severStatus = document.getElementById("curStatus");
    severStatus.innerHTML = "<br>Server Status: Waiting on players"

    //get info related to players
    var minPlayers = msgData.minPlayersNeeded;
    var maxPlayers = msgData.maxPlayersAllowed;
    var curPlayers = ""


    //cast player map json to map and read name of each player
    //also store names in global var
    const players = new Map(Object.entries(msgData.players));
    globalPlayers = new Map();
    for (let [UUID, playerInfo] of players) 
    {
        curPlayers += ("<br>" + playerInfo.name);
        globalPlayers.set(UUID,playerInfo.name);
    }




    //update player status
    playerStatus = document.getElementById("curPlayers"); 
    playerStatus.innerHTML = "Min Players: " + minPlayers + 
                             "<br>Max Players: " + maxPlayers + 
                             "<br><br>Connected Players:" + curPlayers;

}

//update page text to show game starting soon
function update_onStartingSoon(msgData)
{
    //update server status
    severStatus = document.getElementById("curStatus");
    severStatus.innerHTML = "<br>Server Status: Server starting soon" + 
                            "<br>Seconds left: " + msgData.secondsLeft; 

    //get info related to players
    var minPlayers = msgData.minPlayersNeeded;
    var maxPlayers = msgData.maxPlayersAllowed;
    var curPlayers = ""

    //cast player map json to map and read name of each player
    //also store names in global var
    const players = new Map(Object.entries(msgData.players));
    globalPlayers = new Map();
    for (let [UUID, playerInfo] of players) 
    {
        curPlayers += ("<br>" + playerInfo.name)
        globalPlayers.set(UUID,playerInfo.name);
    }

    //update player status
    playerStatus = document.getElementById("curPlayers"); 
    playerStatus.innerHTML = "Min Players: " + minPlayers + 
                             "<br>Max Players: " + maxPlayers + 
                             "<br><br>Connected Players: " + curPlayers;
}

//update page text to show next state of game
function update_onNextState(msgData)
{
    //update server status and number of ticks left
    severStatus = document.getElementById("curStatus");
    severStatus.innerHTML = "<br>Server Status: Game Running" + 
                            "<br><br>Ticks Left: " + msgData.ticksLeft + 
                            "<br>Seconds per Tick: " + msgData.secondsPerTick; 
    

    //iterate through player map and build status string
    var curPlayers = "";
    const players = new Map(Object.entries(msgData.gameState.players));
    for (let [UUID, playerInfo] of players) 
    {
        //get name from global using UUID, then format info string
        var curName = globalPlayers.get(UUID);
        curPlayers += ("<br>-" + curName + ": health=" + playerInfo.health + ", pos=" + playerInfo.row +","+ playerInfo.col); 
    }

    //iterate through player order and build play order string
    var playerOrder = ""; 
    for(let i = 0; i < globalPlayerOrder.length; i++ )
    {
        var curName = globalPlayers.get(globalPlayerOrder[i]);

        //if player is dead we put "(DEAD)" nex to their name
        if(globalPlayersKilled.includes(globalPlayerOrder[i]))
        {
            playerOrder += ("<br>" + (i+1) + ") " + curName + "(DEAD)"); 
        }

        //player is not dead
        else
        {
            playerOrder += ("<br>" + (i+1) + ") " + curName); 
        }
    }

    //update player status
    playerStatus = document.getElementById("curPlayers"); 
    playerStatus.innerHTML =("<br>Player turn order:" + playerOrder + "<br><br>Players remaining in game:" + curPlayers); 


    //iterate through actions taken and build actionsTaken string
    var curActions = ""; 
    const takenActions = new Map(Object.entries(msgData.actionsTaken))
    for(let [UUID, lastAction] of takenActions)
    {
        var curName = globalPlayers.get(UUID);
        var curType = lastAction.type; 
        curActions += ("<br><br>-" + curName + " did action: " + curType);
        
        //add direction if not a weapon drop
        if(lastAction.type != "dropWeapon")
        {
            curActions += (", in direction: " + lastAction.direction); 
        }
    }

    //update actions taken box
    actionStatus = document.getElementById("ActionsTakenText"); 
    actionStatus.innerHTML = ("<br>Actions Taken From Last State:" + curActions); 
}

//update page text to show final game state, and button to refresh viewer
function update_onGameEnd(msgData)
{
    //update server status to show game ended
    severStatus = document.getElementById("curStatus");
    severStatus.innerHTML = "<br>Server Status: Game Ended" + 
                         "<br><br>Ticks Left: 0";
 

    //iterate through player map and build status string
    var curPlayers = "";
    const players = new Map(Object.entries(msgData.gameState.players));
    for (let [UUID, playerInfo] of players) 
    {
        //get name from global using UUID, then format info string
        var curName = globalPlayers.get(UUID);
        curPlayers += ("<br>-" + curName + ": health=" + playerInfo.health + ", pos=" + playerInfo.row +","+ playerInfo.col); 
    }

    //iterate through player order and build play order string
    var playerOrder = ""; 
    for(let i = 0; i < globalPlayerOrder.length; i++ )
    {
        var curName = globalPlayers.get(globalPlayerOrder[i]);

        //if player is dead we put "(DEAD)" nex to their name
        if(globalPlayersKilled.includes(globalPlayerOrder[i]))
        {
            playerOrder += ("<br>" + (i+1) + ") " + curName + "(DEAD)"); 
        }

        //player is not dead
        else
        {
            playerOrder += ("<br>" + (i+1) + ") " + curName); 
        }
    }

    //update player status
    playerStatus = document.getElementById("curPlayers"); 
    playerStatus.innerHTML =("<br>Player turn order:" + playerOrder + "<br><br>Players remaining in game:" + curPlayers); 


    //iterate through actions taken and build actionsTaken string
    var curActions = ""; 
    const takenActions = new Map(Object.entries(msgData.actionsTaken))
    for(let [UUID, lastAction] of takenActions)
    {
        var curName = globalPlayers.get(UUID);
        var curType = lastAction.type; 
        curActions += ("<br><br>-" + curName + " did action: " + curType);
        
        //add direction if not a weapon drop
        if(lastAction.type != "dropWeapon")
        {
            curActions += (", in direction: " + lastAction.direction); 
        }
    }

    //update actions taken box
    actionStatus = document.getElementById("ActionsTakenText"); 
    actionStatus.innerHTML = ("<br>Final Actions Taken:" + curActions); 

    //give player option to refresh page
    var refreshButton = createRefreshButton(); 
    refreshButton.onclick = function()
    {
        location.reload();
    }
}

//update global vars to hold player data and move order as game starts
function update_playerInfo(msgData)
{
    //cast player map json to map and read name of each player
    //store names in global var
    const players = new Map(Object.entries(msgData.players));
    globalPlayers = new Map();
    for (let [UUID, playerInfo] of players) 
    {
        globalPlayers.set(UUID,playerInfo.name);
    }

    //save player order if given
    if(msgData.playerOrder != null)
    {
        //save order of players in global var
        globalPlayerOrder = msgData.playerOrder; 
    }
    
}

//update global var to track new player has died
function update_playerKilled(msgData)
{
    //add UUID of killed player to array
    globalPlayerKilled.push(msgData.id); 
}

//function to display the game arena, takes reference canvas
function updateArena(currentCanvas,mode,canvasWidth,canvasHeight,msgData=null)
{
    //get canvas context and clear
    var curCtx = currentCanvas.getContext("2d");
    curCtx.clearRect(0, 0, canvasWidth, canvasHeight);

    //mode decides what happens next
    switch(mode)
    {
        case "waitingOnServer":
            //fill in background white
            curCtx.fillStyle = "#dbdbdb";
            curCtx.fillRect(0, 0, canvasWidth, canvasHeight);

            //waiting on server to send msg
            curCtx.textAlign = "center";
            curCtx.fillStyle = "black";
            curCtx.font = "60px Arial";
            curCtx.fillText("Waiting on server", canvasWidth/2, canvasHeight/2);
        break; 

        case "waitingOnPlayers":
            //fill in background white
            curCtx.fillStyle = "#dbdbdb";
            curCtx.fillRect(0, 0, canvasWidth, canvasHeight);

            //waiting on players to join game
            curCtx.textAlign = "center";
            curCtx.fillStyle = "black";
            curCtx.font = "60px Arial";
            curCtx.fillText("Waiting on players!", canvasWidth/2, canvasHeight/2);
        break; 

        case "gameStartingSoon": 
            //fill in background white
            curCtx.fillStyle = "#dbdbdb";
            curCtx.fillRect(0, 0, canvasWidth, canvasHeight);

            //game starting soon
            curCtx.textAlign = "center";
            curCtx.fillStyle = "black";
            curCtx.font = "60px Arial";
            curCtx.fillText("Game starting soon!", canvasWidth/2, canvasHeight/2);
        break; 

        case "gameStarting": 
            //fill in background white
            curCtx.fillStyle = "#dbdbdb";
            curCtx.fillRect(0, 0, canvasWidth, canvasHeight);

            //game starting
            curCtx.textAlign = "center";
            curCtx.fillStyle = "black";
            curCtx.font = "60px Arial";
            curCtx.fillText("Game now starting!", canvasWidth/2, canvasHeight/2);
        break; 

        case "gameInitialized": 
            //fill in background white
            curCtx.fillStyle = "#dbdbdb";
            curCtx.fillRect(0, 0, canvasWidth, canvasHeight);

            //game has been init
            curCtx.textAlign = "center";
            curCtx.fillStyle = "black";
            curCtx.font = "60px Arial";
            curCtx.fillText("Game has been initialized!", canvasWidth/2, canvasHeight/2);
        break; 

        case "gameEnded":
            //fill in background white
            curCtx.fillStyle = "#dbdbdb";
            curCtx.fillRect(0, 0, canvasWidth, canvasHeight);

            //text formatting 
            curCtx.textAlign = "center";
            curCtx.fillStyle = "black";
            curCtx.font = "40px Arial";

            //build string to announce winners
            var gameWinners = ""; 
            var winnerArray = msgData.winners; 
            var winLen = winnerArray.length; 

            if(winLen == 0)
            {
                //everyone is dead, no winners
                curCtx.fillText("Game Over", canvasWidth/2, canvasHeight/3);
                curCtx.fillText("There were no winners.", canvasWidth/2, canvasHeight/2);
            }

            else if(winLen == 1)
            {
                //there is only one winner
                curCtx.fillText("Game Over", canvasWidth/2, canvasHeight/3);

                gameWinners = (globalPlayers.get(winnerArray[0]) + " wins!"); 
                curCtx.fillText(gameWinners, canvasWidth/2, canvasHeight/2);
            }

            else if(winLen > 1)
            {
                //there are multiple winners, build string
                curCtx.fillText("Game Over", canvasWidth/2, canvasHeight/3);

                gameWinners = "Winners are: "; 
                for(let i = 0; i < winLen; i++)
                {
                    gameWinners += (globalPlayers.get(winnerArray[i]) + " ");
                }
                curCtx.fillText(gameWinners, canvasWidth/2, canvasHeight/2);
            }

            //write text to canvas
           
    
        break; 

        case "nextState":
            //draw arena using given gameState

            //get width and height of arena, and calculate how large each tile is
            var playField   = msgData.gameState.playfield; 
            var fieldWidth  = playField[0].length; 
            var fieldHeigth = playField.length;

            var tileWidth  = canvasWidth  / fieldWidth; 
            var tileHeigth = canvasHeight / fieldHeigth; 

            //get player and weapon data as map and array
            const players = new Map(Object.entries(msgData.gameState.players));
            const weapons = msgData.gameState.weapons; 

            //draw each tile of the playField
            for(let row = 0; row < fieldHeigth; row++ )
            {
                for(let col = 0; col < fieldWidth; col++)
                {
                    var y_offset = row * tileHeigth; 
                    var x_offset = col * tileWidth; 
                    
                    //draw tile in offset position.
                    //color depends on tile type
                    var curTile = playField[row][col]; 
                    if(curTile == 1)
                    {
                        //tile is a wall
                        curCtx.fillStyle = "black";
                        curCtx.fillRect(x_offset, y_offset, tileWidth, tileHeigth); 
                    }
                    else if(curTile == 0)
                    {
                        //tile is empty
                        curCtx.fillStyle = "#dbdbdb";
                        curCtx.fillRect(x_offset, y_offset, tileWidth, tileHeigth); 

                        //empty tiles may contain a player or a weapon, 
                        //check for each and draw where needed.
                        for (let [UUID, playerInfo] of players) 
                        {
                            if((playerInfo.row -1 == row) && (playerInfo.col -1 == col))
                            {
                                //this tile has a player, draw them
                                curCtx.fillStyle = "blue";
                                curCtx.fillRect(x_offset, y_offset, tileWidth / 2, tileHeigth / 2); 

                                //write player name
                                curCtx.fillStyle = "white";
                                curCtx.font = "15px Arial";
                                curCtx.fillText(globalPlayers.get(UUID), (x_offset + tileWidth / 6), (y_offset + tileHeigth / 3), (tileWidth / 2));
                            }
                        }

                        for (let i = 0; i < weapons.length; i++) 
                        {
                            if((weapons[i].row -1 == row) && (weapons[i].col -1 == col))
                            {
                                //this tile has a weapon, draw it
                                curCtx.fillStyle = "orange";
                                curCtx.fillRect(x_offset, y_offset, tileWidth / 2.5, tileHeigth / 2.5); 

                                //write weapon name
                                curCtx.fillStyle = "white";
                                curCtx.font = "9px Arial";
                                curCtx.fillText(weapons[i].type, (x_offset + tileWidth / 5), (y_offset + tileHeigth / 5), (tileWidth / 2.5));
                            }
                        }
                    }

                }
            }

        break; 
    }
}

//main display logic loop
function runBattlefieldDisplay(addrAndPort,width,height)
{

    //read in connection info from settings.json
    var config = settings;
    
    //only continue if config is non null 
    if(config != null)
    {
        //attempt connection to server, stop if fails
        const socket = connectToGameserver(addrAndPort,config);
        if(socket != null)
        {
            //ready canvas for game
            var battlefieldCanvas = readyCanvas(width,height);
            if(battlefieldCanvas == null)
            {
                alert("Error creating canvas")
            }

            else
            {
                //update canvas to show waiting on server
                updateArena(battlefieldCanvas,"waitingOnServer",width,height);

                //socket and canvas are functional, start reading msg
                socket.onmessage = (msg) =>
                {
                    //part msg and check type
                    const msgData = JSON.parse(msg.data);
                    var type = msgData.type; 

                    switch(type)
                    {
                        case "waitingOnPlayers": 
                            //server is waiting on more player connections
                            updatePage_onWaitingOnPlayers(msgData);
                            updateArena(battlefieldCanvas,"waitingOnPlayers",width,height);
                        break; 

                        case "gameStartingSoon":
                            //game session is starting soon
                            update_onStartingSoon(msgData);
                            updateArena(battlefieldCanvas,"gameStartingSoon",width,height);
                        break; 

                        case "gameStarting":
                            //game is starting
                            //updated player data and move order.
                            update_playerInfo(msgData);
                            //update canvas
                            updateArena(battlefieldCanvas,"gameStarting",width,height);
                        break; 

                        case "init":
                            //no text is currently needed during this state. 
                            //canvas is still updated
                            updateArena(battlefieldCanvas,"gameInitialized",width,height);
                        break; 

                        case "nextState":
                            //game is running.
                            //check if we have player data, if not request from server
                            if(globalPlayers == null || globalPlayerOrder == null)
                            {
                                socket.send(JSON.stringify({ type: "getRegisteredPlayers"}));
                            }

                            //we have player data, update as normal
                            else
                            {
                                //update player moves and player status info
                                update_onNextState(msgData); 
                                //update arena with player positions
                                updateArena(battlefieldCanvas,"nextState",width,height,msgData); 
                            }
                        break; 

                        case "playerKilled":
                            //no action is currently needed during this state. 
                        break; 

                        case "gameEnded":
                            //game has ended
                            //display game results and close socket connection.
                            update_onGameEnd(msgData)
                            updateArena(battlefieldCanvas,"gameEnded",width,height,msgData);
                            socket.close();  
                        break; 

                        case "registeredPlayers":
                            //server has sent list of registered players, update globals
                            update_playerInfo(msgData); 
                        break; 
                    }
                }

                //tell user if connection has error
                socket.onerror = (error) =>
                {
                    //make sure our connection does not result in error
                    console.log(error);
                    alert("Fatal connection error, Reloading page");
                    location.reload();
                }
            }
        }
    }
}


//make start button and listen for click
var startButtonRef = readyGameButtons();
startButtonRef.onclick = function()
{
    //check if user put value in address box
    infoBox= document.getElementById("serverInfoBox");
    var givenAddr = infoBox.value;

    if(givenAddr == "")
    {
        //ask for properinput
        alert("Please enter address and port")
    }
    else
    {
        //remove start UI before canvas is made 
        var startButton = document.getElementById("startGameButton");
        var infoBox= document.getElementById("serverInfoBox");
        var infoBoxText= document.getElementById("infoBoxInstructions");
        startButton.remove();
        infoBox.remove(); 
        infoBoxText.remove(); 

        //call display function
        runBattlefieldDisplay(givenAddr,950,700)
    }
};