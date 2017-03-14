/*
	TODO:
	Tilføj trump tema
*/

var canvas      = document.getElementById('canvas');
canvas.height   = window.innerHeight;
canvas.width    = window.innerWidth;

sideWidth = 230;
var ctx = canvas.getContext('2d');

var isMouseDown = false;

document.getElementById("sideBar").style.width = sideWidth + "px";
superLock = false;


levelBackup = Array();
robotsBackup = Array();

selectedIndex = 0;

parseMap();




var skins = ["Ascii", "Normal", "Trump"];
var currentSkin = 1;
for (var i = 0; i < skins.length; i ++) {
	var iDiv = document.createElement('div');
	iDiv.innerText = skins[i];
	iDiv.id = "skin" + i;
	iDiv.className = 'skin' + ((i == currentSkin) ? " active" : "");
	iDiv.addEventListener('click', changeSkin);
	document.getElementById('skins').appendChild(iDiv);
}

function loadImages(folder) {
	wallImage = new Image;
	wallImage.src = folder + "/wall.png";
	wallImage.onload = function(){
	  drawLevel();
	};
	
	goalImage = new Image;
	goalImage.src = folder + "/goal.png";
	goalImage.onload = function(){
	  drawLevel();
	};
	
	buttonImage = new Image;
	buttonImage.src = folder + "/button.png";
	buttonImage.onload = function(){
	  drawLevel();
	};
	
	leverImage = new Image;
	leverImage.src = folder + "/lever.png";
	leverImage.onload = function(){
	  drawLevel();
	};
	
	botImage = new Image;
	botImage.src = folder + "/bot.png";
	botImage.onload = function(){
	  drawLevel();
	};
}

function drawLevel() {
	resetCanvas();
	
	ctx.font = (.75 * sqaureSize) + "px Comic Sans MS";
	ctx.textBaseline = "middle";
	ctx.textAlign = "center";
	
	for (var x = 1; x < level.length; x ++) {
		ctx.beginPath();
		ctx.moveTo(sideWidth + x * sqaureSize, 0);
		ctx.lineTo(sideWidth + x * sqaureSize, canvas.height);
		ctx.stroke();
		
		ctx.beginPath();
		ctx.moveTo(sideWidth + 0, x * sqaureSize);
		ctx.lineTo(sideWidth + canvas.width, x * sqaureSize);
		ctx.stroke();
	}
	var wires = Array();
	
	for (var x = 0; x < level.length; x ++) {
		for (var y = 0; y < level[x].length; y ++) {
			if (currentSkin != 0) {
				if (level[x][y] == "#") {
					ctx.drawImage(wallImage, sideWidth + x * sqaureSize, y * sqaureSize, sqaureSize, sqaureSize);
				} else if (level[x][y] == "!") {
					ctx.drawImage(goalImage, sideWidth + x * sqaureSize, y * sqaureSize, sqaureSize, sqaureSize);
				} else if (level[x][y] >= 'a' && level[x][y] <= 'z') {
					ctx.drawImage(buttonImage, sideWidth + x * sqaureSize, y * sqaureSize, sqaureSize, sqaureSize);
				} else if (level[x][y] >= 'A' && level[x][y] <= 'Z') {
					ctx.drawImage(leverImage, sideWidth + x * sqaureSize, y * sqaureSize, sqaureSize, sqaureSize);
				}
			} else {
				ctx.fillStyle = "black";
				ctx.fillText(level[x][y], sideWidth + x * sqaureSize + sqaureSize / 2, y * sqaureSize + sqaureSize / 2); 
			}
			if (level[x][y] >= 'a' && level[x][y] <= 'z') {
				wires.push([[x, y], getPositionOfButtonLeverWall(level[x][y])]);
			} else if (level[x][y] >= 'A' && level[x][y] <= 'Z') {
				wires.push([[x, y], getPositionOfButtonLeverWall(level[x][y])]);
			}
		}
	}
	for (var i = 0; i < numberRobots; i ++) {
		var x = robots[i][0];
		var y = robots[i][1];
		ctx.fillStyle = "red";
		if (i == selectedRobot) {
			ctx.fillStyle = "blue";
		}
		if (currentSkin != 0) {
			ctx.drawImage(botImage, sideWidth + x * sqaureSize, y * sqaureSize, sqaureSize, sqaureSize);
		}
		ctx.fillText(i, sideWidth + x * sqaureSize + sqaureSize / 2, y * sqaureSize + sqaureSize / 2); 
	}
	for (var x = 0; x < level.length; x ++) {
		for (var y = 0; y < level[x].length; y ++) {
			if (currentSkin != 0) {
				if (level[x][y] == "!") {
					ctx.drawImage(goalImage, sideWidth + x * sqaureSize, y * sqaureSize, sqaureSize, sqaureSize);
				}
			}
		}
	}
	
	// wires
	var wireColors = ["red", "green", "blue"];
	var wireWidth = sqaureSize / 30;
	for (var i = 0; i < wires.length; i ++) {
		var x1 = wires[i][0][0];
		var y1 = wires[i][0][1];
		var x2 = wires[i][1][0];
		var y2 = wires[i][1][1];
		var offset = wireWidth * 4 + (i * wireWidth * 2) % (sqaureSize - wireWidth * 8);
		ctx.fillStyle = wireColors[i % wireColors.length];
		
		
		ctx.beginPath();
	    ctx.arc(sideWidth + x1 * sqaureSize + offset, y1 * sqaureSize + offset, wireWidth * 2, 0, 2 * Math.PI, false);
	    ctx.fill();
	    ctx.lineWidth = wireWidth / 2;
	    ctx.strokeStyle = '#003300';
	    ctx.stroke();
	    
	    ctx.beginPath();
	    ctx.arc(sideWidth + x2 * sqaureSize + offset, y2 * sqaureSize + offset, wireWidth * 2, 0, 2 * Math.PI, false);
	    ctx.fill();
	    ctx.lineWidth = wireWidth / 2;
	    ctx.strokeStyle = '#333333';
	    ctx.stroke();
	    
	    
		ctx.strokeStyle = wireColors[i % wireColors.length];
		ctx.lineWidth = wireWidth;
	    
		ctx.beginPath();
		ctx.moveTo(sideWidth + x1 * sqaureSize + offset, y1 * sqaureSize + offset);
		ctx.lineTo(sideWidth + x1 * sqaureSize + offset, y2 * sqaureSize + offset);
		ctx.lineTo(sideWidth + x2 * sqaureSize + offset, y2 * sqaureSize + offset);
		ctx.stroke();
	}
}

function getPositionOfButtonLeverWall(a) {
	if (a >= 'a' && a <= 'z') {
		for (var i = 0; i < buttons.length; i ++) {
			if (buttons[i][0] == a) {
				return [buttons[i][1], buttons[i][2]];
			}
		}
	} else if (a >= 'A' && a <= 'Z') {
		for (var i = 0; i < levers.length; i ++) {
			if (levers[i][0] == a) {
				return [levers[i][1], levers[i][2]];
			}
		}
	}
}

function performActionAtPosition(a, b) {
	if (a >= 'a' && a <= 'z') {
		for (var i = 0; i < buttons.length; i ++) {
			if (buttons[i][0] == a) {
				level[buttons[i][1]][buttons[i][2]] = ' ';
			}
		}
	} else if (a >= 'A' && a <= 'Z') {
		for (var i = 0; i < levers.length; i ++) {
			if (levers[i][0] == a) {
				if (level[levers[i][1]][levers[i][2]] == '#') {
					level[levers[i][1]][levers[i][2]] = ' ';
				} else if (!robotAtPos(levers[i][1], levers[i][2], -1)) {
					level[levers[i][1]][levers[i][2]] = '#';
				}
			}
		}
	}
	if (a == '!') {
		b = b || 0;
		if (b == 1) {
			alert("Yaaaah!! you did it in only " + (document.getElementById("movesBox").children.length - 1) + " moves");
		}
	}
}

function dePerformActionAtPosition(a) {
	if (a >= 'a' && a <= 'z') {
		for (var i = 0; i < buttons.length; i ++) {
			if (buttons[i][0] == a) {
				level[buttons[i][1]][buttons[i][2]] = '#';
			}
		}
	}
}

function robotAtPos(x, y, ignore) {
	for (var i = 0; i < numberRobots; i ++) {
		if (x == robots[i][0] && y == robots[i][1] && ignore != i) {
			return true;
		} 
	}
	return false;
}

document.onkeydown = checkKey;
function checkKey(e, b, instant) {
	b = b || false;
	instant = instant || false;
	if (locked || superLock) {
		return;
	}
    e = e || window.event;
    
    if (e.keyCode >= '37' && e.keyCode <= '40') {
	    var pos = [robots[selectedRobot][0], robots[selectedRobot][1]];
	    var dir = "";
	    if (e.keyCode == '38') {
	        // up arrow
	        robots[selectedRobot][1] --;
	        dir = "U";
	    } else if (e.keyCode == '40') {
	        // down arrow
	        robots[selectedRobot][1] ++;
	        dir = "D";
	    } else if (e.keyCode == '37') {
	       // left arrow
	        robots[selectedRobot][0] --;
	        dir = "L";
	    } else if (e.keyCode == '39') {
	       // right arrow
	        robots[selectedRobot][0] ++;
	        dir = "R";
	    }
	    var conf = true;
	    if (document.getElementById("movesBox").children.length - 1 != selectedIndex && !b) {
		    conf = confirm("Are you sure you want to move? This will delete all moves after the current move.");
	    }
	    var newSpot = "#";
	    var newPos = robots[selectedRobot];
	    if (newPos[0] < 0 || newPos[1] < 0 || newPos[0] > width - 1 || newPos[1] > height - 1) {
		    conf = false;
	    } else {
		    newSpot = level[robots[selectedRobot][0]][robots[selectedRobot][1]];
	    }
	    if (newSpot == '#' || robotAtPos(robots[selectedRobot][0], robots[selectedRobot][1], selectedRobot) || !conf) {
		    robots[selectedRobot] = pos;
	    } else {
		    
		    if (!b) {
			    levelBackup = levelBackup.slice(0, selectedIndex + 1);
			    robotsBackup = robotsBackup.slice(0, selectedIndex + 1);
			    for (var i = selectedIndex + 1; i < document.getElementById("movesBox").children.length; i ++) {
				    var el = document.getElementById("movesBox").children[i];
				    console.log(el.innerHTML);
				    document.getElementById("movesBox").removeChild(el);
				    i --;
			    }
			    
			    var option = document.createElement("li");
			    option.appendChild(document.createTextNode(((document.getElementById("movesBox").children.length) + ": ") + selectedRobot + dir));
			    option.addEventListener('click', OnMoveChanged);
			    document.getElementById("movesBox").appendChild(option);
		        dePerformActionAtPosition(level[pos[0]][pos[1]]);
			    performActionAtPosition(level[robots[selectedRobot][0]][robots[selectedRobot][1]]);
			    
				levelBackup.push(deepCopyxDimArray(level));
				robotsBackup.push(deepCopyxDimArray(robots));
			    OnMoveChanged(document.getElementById("movesBox").children.length - 1);
			    performActionAtPosition(level[robots[selectedRobot][0]][robots[selectedRobot][1]]);
			    
			    
			    
			    
			    
			    dePerformActionAtPosition(level[robots[selectedRobot][0]][robots[selectedRobot][1]]);
			    performActionAtPosition(level[pos[0]][pos[1]]);
			    performActionAtPosition(level[pos[0]][pos[1]]);
		    } else {
			    OnMoveChanged(selectedIndex);
			    performActionAtPosition(level[robots[selectedRobot][0]][robots[selectedRobot][1]]);
		    }
		    if (!instant) {
		    	animateMove(deepCopyxDimArray(pos), deepCopyxDimArray(robots[selectedRobot]), 15, b);
		    } else {
			    dePerformActionAtPosition(level[pos[0]][pos[1]]);
			    performActionAtPosition(level[robots[selectedRobot][0]][robots[selectedRobot][1]]);
		    }
		    
	    }
    }
    
    if (e.keyCode < 48 + numberRobots && e.keyCode >= 48) {
	    changeSelectedRobot(e.keyCode - 48);
    }
    drawLevel();
}

locked = false;

function animateMove(fromPos, toPos, stepsBack) {
	locked = true;
    robots[selectedRobot][0] = (fromPos[0] * stepsBack + toPos[0] * (15 - stepsBack)) / 15.0;
    robots[selectedRobot][1] = (fromPos[1] * stepsBack + toPos[1] * (15 - stepsBack)) / 15.0;
    drawLevel();
    if (stepsBack == 7) {
	    dePerformActionAtPosition(level[fromPos[0]][fromPos[1]]);
		performActionAtPosition(level[toPos[0]][toPos[1]], 1);
    }
    if (stepsBack >= 0) {
	    timer = setTimeout(function () {
		  animateMove(fromPos, toPos, stepsBack - 1);
		}, 16);
    } else {
	    robots[selectedRobot] = toPos;
	    locked = false;
    }
    
}

function changeSelectedRobotButton(e) {
	changeSelectedRobot(parseInt(e.target.innerText));
}

function changeSelectedRobot(n) {
	document.getElementById("number" + selectedRobot).className = "number";
    selectedRobot = n;
    document.getElementById("number" + selectedRobot).className = "number active";
    drawLevel();
}

function changeSkin(e) {
	e = e.target;
	var n = parseInt(e.id.replace("skin", ""));
	document.getElementById("skin" + currentSkin).className = "skin";
    currentSkin = n;
    document.getElementById("skin" + currentSkin).className = "skin active";
    drawLevel();
    if (currentSkin == 1) {
	    loadImages("default");
    } else {
	    loadImages("trump");
    }
}

loadImages("default");

canvas.addEventListener('mousedown', function mouseDown(e) {
	
});

canvas.addEventListener('mousemove', function mouseDown(e) {
	
});

canvas.addEventListener('mouseup', function mouseDown(e) {
	
});

function resetCanvas() {
	canvas.height   = window.innerHeight;
	canvas.width    = window.innerWidth;
}



function makeAutoMove() {
	if (moves.length >= 2) {
		superLock = true;
		nextRobot = moves.charAt(0);
		nextDirection = moves.charAt(1);
		moves = moves.substring(2);
		selectedRobot = nextRobot;
		keyCode = 0;
		if (nextDirection == "U") {
	        // up arrow
	        keyCode = 38;
	    } else if (nextDirection == "D") {
	        // down arrow
	        keyCode = 40;
	    } else if (nextDirection == "L") {
	       // left arrow
	        keyCode = 37;
	    } else if (nextDirection == "R") {
	       // right arrow
	        keyCode = 39;
	    }
	    var e = {};
	    e["keyCode"] = keyCode;
	    superLock = false;
	    var le = document.getElementById("movesBox").children.length;
		checkKey(e);
		if (le == document.getElementById("movesBox").children.length) {
			var option = document.createElement("li");
		    option.appendChild(document.createTextNode(((document.getElementById("movesBox").children.length) + ": ") + nextRobot + nextDirection));
		    option.addEventListener('click', OnMoveChanged);
		    document.getElementById("movesBox").appendChild(option);;
			levelBackup.push(deepCopyxDimArray(level));
			robotsBackup.push(deepCopyxDimArray(robots));
			OnMoveChanged(le);
		}
		superLock = true;
	    timer = setTimeout(makeAutoMove, 500);
    } else {
	    superLock = false;
    }
}

makeAutoMove();

function parseMap() {
	if (map.length > 0) {
		console.log(map);
		var lines = map.split('\n');
		width = parseInt(lines[0].split(" ")[0]);
		height = parseInt(lines[0].split(" ")[1]);
		numberRobots = parseInt(lines[1]);
		var numberOfLevers = parseInt(lines[2].split(" ")[0]);
		var numberOfButtons = parseInt(lines[2].split(" ")[1]);
		
		robots = new Array(numberRobots);
		level = new Array();
		for (var i = 0; i < height; i ++) {
			var temp = new Array();
			for (var j = 0; j < width; j ++) {
				var c = lines[3 + i].charAt(j);
				temp.push(c);
				var charCode = c.charCodeAt(0) & 255;
				if (charCode >= 48 && charCode <= 57) {
					robots[charCode - 48] = [j, i];
				}
			}
			level.push(temp);
		}
		
		levers = new Array(numberOfLevers);
		for (var i = 0; i < numberOfLevers; i ++) {
			var c = lines[3 + height + i].split(" ")[0].charCodeAt(0);
			var x = parseInt(lines[3 + height + i].split(" ")[1]);
			var y = parseInt(lines[3 + height + i].split(" ")[2]);
			levers[c - 65] = [String.fromCharCode(c).charAt(0), x, y];
		}
		
		buttons = new Array(numberOfButtons);
		for (var i = 0; i < numberOfButtons; i ++) {
			var c = lines[3 + height + numberOfLevers + i].split(" ")[0].charCodeAt(0);
			var x = parseInt(lines[3 + height + numberOfLevers + i].split(" ")[1]);
			var y = parseInt(lines[3 + height + numberOfLevers + i].split(" ")[2]);
			buttons[c - 97] = [String.fromCharCode(c).charAt(0), x, y];
		}
	} else {
		level = [
		  ['#', '#', '#', '#', '#', '#', '#', '#'],
		  ['#', 'b', '#', ' ', '#', '#', ' ', '#'],
		  ['#', 'a', '#', '#', '#', '#', ' ', '#'],
		  ['#', ' ', '#', '#', ' ', '#', ' ', '#'],
		  ['#', ' ', '#', ' ', '#', '#', ' ', '#'],
		  ['#', ' ', '#', '#', '#', '#', 'B', '#'],
		  ['#', ' ', '#', ' ', '!', '#', 'A', '#'],
		  ['#', '#', '#', '#', '#', '#', '#', '#']
		];
		map = "8 8\n3\n2 2\n########\n#b#0##1#\n#a#### #\n# ## # #\n# # ## #\n# ####B#\n#2# !#A#\n########\nA 4 4\nB 4 2\na 3 5\nb 4 1";
		robots = [[3, 1], [6, 1], [1, 6]];
		levers = [['A', 4, 4], ['B', 4, 2]];
		buttons = [['a', 3, 5], ['b', 4, 1]];
		numberRobots = 3;
		width = level[0].length
		height = level.length

	}
	
	
	ajustSize();
	
	

	level = level[0].map(function(col, i) { 
	  return level.map(function(row) { 
	    return row[i] 
	  })
	});
	
	levelBackup.push(deepCopyxDimArray(level));
	robotsBackup.push(deepCopyxDimArray(robots));
	
	selectedRobot = 0;
	
	document.getElementById('numbers').innerHTML = "";
	for (var i = 0; i < numberRobots; i ++) {
    	var iDiv = document.createElement('div');
    	iDiv.innerText = i;
    	iDiv.id = "number" + i;
    	iDiv.className = 'number' + ((i == 0) ? " active" : "");
    	iDiv.addEventListener('click', changeSelectedRobotButton);
    	document.getElementById('numbers').appendChild(iDiv);
    }
}

function ajustSize() {
	sideWidth = 230;
	heightPerBlock = window.innerHeight / height;
	widthPerBlock = (window.innerWidth - sideWidth) / width;
	
	sqaureSize = Math.min(widthPerBlock, heightPerBlock)
	
	bufferSize = (window.innerWidth - sideWidth - sqaureSize * width) / 2;
	
	document.getElementById("leftBuffer").style.width = bufferSize + "px";
	document.getElementById("rightBuffer").style.width = bufferSize + "px";
	
	bufferHeight = window.innerHeight - height * sqaureSize;
	document.getElementById("buttomBuffer").style.height = bufferHeight + "px";
	document.getElementById("buttomBuffer").style.width = window.innerWidth + "px";
	
	document.getElementById("sideBar").style.left = bufferSize + "px";
	document.getElementById("rightBuffer").style.left = bufferSize + sideWidth + width * sqaureSize + "px";
	document.getElementById("buttomBuffer").style.top = height * sqaureSize + "px";
	sideWidth += bufferSize
	
	timer = setTimeout(ajustSize, 100);
	
	
	try {
	    drawLevel();
	}
	catch(err) {
	}
}

function scrollUL(li, first) {
    // scroll UL to make li visible
    // li can be the li element or its id
    if (typeof li !== "object"){
        li = document.getElementById(li);
    }
    var ul = li.parentNode;
    // fudge adjustment for borders effect on offsetHeight
    var fudge = 4;
    // bottom most position needed for viewing
    var bottom = (ul.scrollTop + (ul.offsetHeight - fudge) - li.offsetHeight);
    // top most position needed for viewing
    var top = ul.scrollTop + fudge;
    if (li.offsetTop <= top){
        // move to top position if LI above it
        // use algebra to subtract fudge from both sides to solve for ul.scrollTop
        ul.scrollTop = li.offsetTop - fudge;
    } else if (li.offsetTop >= bottom) {
        // move to bottom position if LI below it
        // use algebra to subtract ((ul.offsetHeight - fudge) - li.offsetHeight) from both sides to solve for ul.scrollTop
        ul.scrollTop = li.offsetTop - ((ul.offsetHeight - fudge) - li.offsetHeight) ;
    }
    
    if (first) {
	    timer = scrollUL(li, false);
    }
};

function OnMoveChanged(value){
	if (value.constructor === MouseEvent) {
		value = value.target.innerHTML;
		value = parseInt(value.split(":")[0]);
	}
	if (value.constructor === String) {
		value = 0;
	}
	var s = document.getElementById("movesBox").children;
	for (var i = 0; i < s.length; i ++) {
		if (i <= value) {
			s[i].className = "selected";
		} else {
			s[i].className = "";
		}
	}
	scrollUL(s[value], true);
	selectedIndex = value;
	level = deepCopyxDimArray(levelBackup[value]);
	robots = deepCopyxDimArray(robotsBackup[value]);
}

function deepCopyxDimArray(a) {
	if (!(a.constructor === Array)) {
		return a;
	} else {
		var newA = Array();
		for (var i = 0; i < a.length; i ++) {
			newA.push(deepCopyxDimArray(a[i]));
		}
		return newA;
	}
}

function clearLevel() {
	if (locked || superLock) {
		return;
	}
	OnMoveChanged(0);
	selectedIndex = 0;
	levelBackup = levelBackup.slice(0, selectedIndex + 1);
    robotsBackup = robotsBackup.slice(0, selectedIndex + 1);
    for (var i = selectedIndex + 1; i < document.getElementById("movesBox").children.length; i ++) {
	    var el = document.getElementById("movesBox").children[i];
	    console.log(el.innerHTML);
	    document.getElementById("movesBox").removeChild(el);
	    i --;
    }
}

function loadBoardFile() {
	clearLevel();
	map = document.getElementById('boardField').value;
	parseMap();
	document.getElementById('boardBoxPop').style.visibility = 'hidden';
}

function nextMove() {
	if (document.getElementById("movesBox").children.length - 1 != selectedIndex) {
		var move = document.getElementById("movesBox").children[selectedIndex + 1].innerHTML.split(": ")[1];
		selectedIndex ++;
		nextRobot = move.charAt(0);
		nextDirection = move.charAt(1);
		selectedRobot = nextRobot;
		keyCode = 0;
		if (nextDirection == "U") {
	        // up arrow
	        keyCode = 38;
	    } else if (nextDirection == "D") {
	        // down arrow
	        keyCode = 40;
	    } else if (nextDirection == "L") {
	       // left arrow
	        keyCode = 37;
	    } else if (nextDirection == "R") {
	       // right arrow
	        keyCode = 39;
	    }
	    var e = {};
	    e["keyCode"] = keyCode;
		checkKey(e, true);
		return true;
	} else {
		return false;
	}
}

function previousMove() {
	if (0 != selectedIndex) {
		OnMoveChanged(selectedIndex - 1);
	}
}

function playLevel() {
	superLock = false;
	if (nextMove()) {
		superLock = true;
		timer = setTimeout(playLevel, 500);
	} else {
		superLock = false;
	}
}

function getMovesString() {
	var s = "";
	for (var i = 1; i < document.getElementById("movesBox").children.length; i ++) {
		s += document.getElementById("movesBox").children[i].innerHTML.split(": ")[1] + "\n";
	}
	return s;
}

function parseMovesString() {
	clearLevel();
	moves = document.getElementById('movesField').value.split("\n").join("");
	while (moves.length >= 2) {
		console.log(moves);
		nextRobot = moves.charAt(0);
		nextDirection = moves.charAt(1);
		moves = moves.substring(2);
		selectedRobot = nextRobot;
		keyCode = 0;
		if (nextDirection == "U") {
	        // up arrow
	        keyCode = 38;
	    } else if (nextDirection == "D") {
	        // down arrow
	        keyCode = 40;
	    } else if (nextDirection == "L") {
	       // left arrow
	        keyCode = 37;
	    } else if (nextDirection == "R") {
	       // right arrow
	        keyCode = 39;
	    }
	    var e = {};
	    e["keyCode"] = keyCode;
		checkKey(e, false, true);
	}
	OnMoveChanged(0);
	document.getElementById('moveBoxPop').style.visibility = 'hidden';
}

function openURL() {
	console.log(map.split(" ").join("%20").split("#").join("%23"));
	window.open("http://www.319.dk/MR/?map=" + map.split(" ").join("%20").split("#").join("%23").split("\n").join("\\n") + "&moves=" + getMovesString(),'_blank');
}