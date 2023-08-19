# Basic chess engine
import chess

pieceSquareTable = [
[ -50,-40,-30,-30,-30,-30,-40,-50 ],
[ -40,-20, 0, 5, 5, 0,-20,-40 ],
[ -30, 0, 10, 15, 15, 10, 0,-30 ],
[ -30, 5, 15, 20, 20, 15, 5,-30 ],
[ -30, 5, 15, 20, 20, 15, 5,-30 ],
[ -30, 0, 10, 15, 15, 10, 0,-30 ],
[ -40,-20, 0, 5, 5, 0,-20,-40 ],
[ -50,-40,-30,-30,-30,-30,-40,-50 ] ]

# Simple evaluation function
# Add up values of pieces, weighted by position (centralized is better)
def eval(board):
    scoreWhite = 0
    scoreBlack = 0
    for i in range (0,8):
        for j in range (0,8):
            squareIJ = chess.square(i,j)
            pieceIJ = board.piece_at(squareIJ)
            if str(pieceIJ) == "P":
                scoreWhite += (100 + pieceSquareTable[i][j])
            elif str(pieceIJ) == "N":
                scoreWhite += (310 + pieceSquareTable[i][j])
            elif str(pieceIJ) == "B":
                scoreWhite += (320 + pieceSquareTable[i][j])
            elif str(pieceIJ) == "R":
                scoreWhite += (500 + pieceSquareTable[i][j])
            elif str(pieceIJ) == "Q":
                scoreWhite += (900 + pieceSquareTable[i][j])
            elif str(pieceIJ) == "p":
                scoreBlack += (100 + pieceSquareTable[i][j])
            elif str(pieceIJ) == "n":
                scoreBlack += (310 + pieceSquareTable[i][j])
            elif str(pieceIJ) == "b":
                scoreBlack += (320 + pieceSquareTable[i][j])
            elif str(pieceIJ) == "r":
                scoreBlack += (500 + pieceSquareTable[i][j])
            elif str(pieceIJ) == "q":
                scoreBlack += (900 + pieceSquareTable[i][j])
    return scoreWhite - scoreBlack

# Minimax algorithm
# Record number of nodes visited
def minimax(board , depth , maximize):
    if(board.is_checkmate()):
        if(board.turn == chess.WHITE):
            return -10000, 1
        else:
            return 10000, 1
    if(board.is_stalemate () or board.is_insufficient_material ()):
        return 0, 1
    if depth == 0:
        return eval(board), 1
    if(maximize):
        best_value = -99999
        nodes_visited = 0
        for move in board.legal_moves:
            board.push(move)
            val, nodes = minimax(board , depth -1, not maximize)
            best_value = max(best_value , val)
            board.pop()
            nodes_visited += nodes
        return best_value, nodes_visited
    else:
        best_value = 99999
        nodes_visited = 0
        for move in board.legal_moves:
            board.push(move)
            val, nodes = minimax(board , depth -1, not maximize)
            best_value = min(best_value , val)
            board.pop()
            nodes_visited += nodes
        return best_value, nodes_visited

# Move generator
def getNextMove(depth , board , maximize):
    legals = board.legal_moves
    bestMove = None
    bestValue = -99999
    nodes_visited = 0
    if(not maximize):
        bestValue = 99999
    for move in legals:
        board.push(move)
        value, nodes = minimax(board , depth - 1, (not maximize))
        board.pop()
        if maximize:
            if value > bestValue:
                bestValue = value
                bestMove = move
        else:
            if value < bestValue:
                bestValue = value
                bestMove = move
        nodes_visited += nodes
    return (bestMove , bestValue, nodes_visited)


# Test
#board = chess.Board()
board = chess.Board("r1bqkb1r/pppp1ppp/2n2n2/4p2Q/2B1P3/8/PPPP1PPP/RNB1K1NR w KQkq - 4 4")
print(board)
print(" current evaluation ")
print(eval(board))
print(" move generator ")
print(board.legal_moves.count())
print(getNextMove(1 , board , True))
print(getNextMove(2 , board , True))
print(getNextMove(3 , board , True))