#!/usr/bin/env python3
"""
Chess Training Data Collection

This script helps download and prepare chess game data for neural network training.
It can fetch games from various sources and convert them to training format.
"""

import requests
import gzip
import os
import chess.pgn
from pathlib import Path
import argparse
import logging
from typing import List, Optional
from tqdm import tqdm

# Set up logging
logging.basicConfig(level=logging.INFO, format='%(asctime)s - %(levelname)s - %(message)s')
logger = logging.getLogger(__name__)


class DataCollector:
    """Collects and prepares chess training data"""
    
    def __init__(self, data_dir: str = "data"):
        self.data_dir = Path(data_dir)
        self.data_dir.mkdir(exist_ok=True)
    
    def download_lichess_database(self, year: int, month: int, variant: str = "standard") -> str:
        """
        Download Lichess database for a specific month
        
        Args:
            year: Year (e.g., 2023)
            month: Month (1-12)
            variant: Game variant ("standard", "blitz", "rapid", etc.)
            
        Returns:
            Path to downloaded PGN file
        """
        # Lichess database URL pattern
        filename = f"lichess_db_{variant}_{year}-{month:02d}.pgn.bz2"
        url = f"https://database.lichess.org/{variant}/lichess_db_{variant}_{year}-{month:02d}.pgn.bz2"
        
        output_path = self.data_dir / filename
        pgn_path = self.data_dir / filename.replace('.bz2', '')
        
        if pgn_path.exists():
            logger.info(f"PGN file already exists: {pgn_path}")
            return str(pgn_path)
        
        if not output_path.exists():
            logger.info(f"Downloading {url}")
            try:
                response = requests.get(url, stream=True)
                response.raise_for_status()
                
                total_size = int(response.headers.get('content-length', 0))
                
                with open(output_path, 'wb') as f:
                    with tqdm(total=total_size, unit='B', unit_scale=True, desc="Downloading") as pbar:
                        for chunk in response.iter_content(chunk_size=8192):
                            if chunk:
                                f.write(chunk)
                                pbar.update(len(chunk))
                
                logger.info(f"Downloaded: {output_path}")
            except Exception as e:
                logger.error(f"Failed to download {url}: {e}")
                return None
        
        # Extract bz2 file
        logger.info(f"Extracting {output_path}")
        try:
            import bz2
            with bz2.open(output_path, 'rt', encoding='utf-8') as f_in:
                with open(pgn_path, 'w', encoding='utf-8') as f_out:
                    with tqdm(desc="Extracting") as pbar:
                        while True:
                            chunk = f_in.read(8192)
                            if not chunk:
                                break
                            f_out.write(chunk)
                            pbar.update(len(chunk))
            
            # Remove compressed file to save space
            output_path.unlink()
            logger.info(f"Extracted to: {pgn_path}")
            
        except Exception as e:
            logger.error(f"Failed to extract {output_path}: {e}")
            return None
        
        return str(pgn_path)
    
    def create_sample_dataset(self, pgn_path: str, output_path: str, max_games: int = 10000) -> str:
        """
        Create a smaller sample dataset from a large PGN file
        
        Args:
            pgn_path: Path to source PGN file
            output_path: Path for sample output
            max_games: Maximum number of games to include
            
        Returns:
            Path to sample PGN file
        """
        logger.info(f"Creating sample dataset with {max_games} games")
        
        games_written = 0
        with open(pgn_path, 'r', encoding='utf-8', errors='ignore') as f_in:
            with open(output_path, 'w', encoding='utf-8') as f_out:
                while games_written < max_games:
                    game = chess.pgn.read_game(f_in)
                    if game is None:
                        break
                    
                    # Write game to sample file
                    print(game, file=f_out)
                    print("", file=f_out)  # Empty line between games
                    
                    games_written += 1
                    
                    if games_written % 1000 == 0:
                        logger.info(f"Written {games_written} games")
        
        logger.info(f"Sample dataset created: {output_path} ({games_written} games)")
        return output_path
    
    def filter_games_by_criteria(self, pgn_path: str, output_path: str, 
                                min_elo: int = 1800, min_moves: int = 20, 
                                max_moves: int = 100) -> str:
        """
        Filter games by quality criteria
        
        Args:
            pgn_path: Source PGN file
            output_path: Filtered output file
            min_elo: Minimum player rating
            min_moves: Minimum number of moves
            max_moves: Maximum number of moves
            
        Returns:
            Path to filtered PGN file
        """
        logger.info(f"Filtering games (min_elo={min_elo}, moves={min_moves}-{max_moves})")
        
        games_total = 0
        games_kept = 0
        
        with open(pgn_path, 'r', encoding='utf-8', errors='ignore') as f_in:
            with open(output_path, 'w', encoding='utf-8') as f_out:
                while True:
                    game = chess.pgn.read_game(f_in)
                    if game is None:
                        break
                    
                    games_total += 1
                    
                    if self.meets_criteria(game, min_elo, min_moves, max_moves):
                        print(game, file=f_out)
                        print("", file=f_out)
                        games_kept += 1
                    
                    if games_total % 5000 == 0:
                        logger.info(f"Processed {games_total} games, kept {games_kept}")
        
        logger.info(f"Filtering complete: kept {games_kept}/{games_total} games ({games_kept/games_total*100:.1f}%)")
        return output_path
    
    def meets_criteria(self, game, min_elo: int, min_moves: int, max_moves: int) -> bool:
        """Check if game meets quality criteria"""
        headers = game.headers
        
        # Check ratings
        try:
            white_elo = int(headers.get("WhiteElo", "0"))
            black_elo = int(headers.get("BlackElo", "0"))
            if white_elo < min_elo or black_elo < min_elo:
                return False
        except (ValueError, TypeError):
            return False
        
        # Check game length
        moves = list(game.mainline_moves())
        if len(moves) < min_moves or len(moves) > max_moves:
            return False
        
        # Check result is decisive or draw (not abandoned)
        result = headers.get("Result", "*")
        if result not in ["1-0", "0-1", "1/2-1/2"]:
            return False
        
        # Check time control (avoid ultra-bullet)
        time_control = headers.get("TimeControl", "")
        if time_control and "+" in time_control:
            try:
                base_time = int(time_control.split("+")[0])
                if base_time < 180:  # Less than 3 minutes
                    return False
            except:
                pass
        
        return True
    
    def download_sample_games(self) -> str:
        """Download a small sample of games for testing"""
        sample_pgns = [
            "https://raw.githubusercontent.com/official-stockfish/books/master/8moves_v3.pgn",
            # Add more sample PGN URLs here
        ]
        
        sample_file = self.data_dir / "sample_games.pgn"
        
        if sample_file.exists():
            logger.info(f"Sample file already exists: {sample_file}")
            return str(sample_file)
        
        with open(sample_file, 'w') as f_out:
            for url in sample_pgns:
                try:
                    logger.info(f"Downloading sample from {url}")
                    response = requests.get(url)
                    response.raise_for_status()
                    f_out.write(response.text)
                    f_out.write("\n\n")
                except Exception as e:
                    logger.warning(f"Failed to download {url}: {e}")
        
        # If that doesn't work, create a minimal example
        if not sample_file.exists() or sample_file.stat().st_size < 1000:
            self.create_minimal_sample(str(sample_file))
        
        return str(sample_file)
    
    def create_minimal_sample(self, output_path: str):
        """Create a minimal sample for testing"""
        sample_games = [
            """[Event "Sample Game 1"]
[Site "Training"]
[Date "2023.01.01"]
[Round "1"]
[White "Player1"]
[Black "Player2"]
[Result "1-0"]
[WhiteElo "2000"]
[BlackElo "1950"]
[TimeControl "600+5"]

1. e4 e5 2. Nf3 Nc6 3. Bb5 a6 4. Ba4 Nf6 5. O-O Be7 6. Re1 b5 7. Bb3 d6 8. c3 O-O 9. h3 Nb8 10. d4 Nbd7 11. Nbd2 Bb7 12. Bc2 Re8 13. Nf1 Bf8 14. Ng3 g6 15. a4 c5 16. d5 c4 17. Be3 Nc5 18. Qd2 h6 19. Bxc5 dxc5 20. axb5 axb5 21. Rxa8 Qxa8 22. Re3 Qa5 23. Nf5 Qxd2 24. Nxd2 Bc8 25. f4 exf4 26. Rf3 Ne4 27. Nxe4 f5 28. Nc3 Rxe4 29. Nxb5 Re1+ 30. Kh2 Bd7 31. Nxc7 1-0""",
            
            """[Event "Sample Game 2"]
[Site "Training"]
[Date "2023.01.02"]
[Round "1"]
[White "Player3"]
[Black "Player4"]
[Result "0-1"]
[WhiteElo "1900"]
[BlackElo "2050"]
[TimeControl "900+10"]

1. d4 d5 2. c4 c6 3. Nf3 Nf6 4. Nc3 dxc4 5. a4 Bf5 6. e3 e6 7. Bxc4 Bb4 8. O-O Nbd7 9. Qe2 Bg6 10. e4 O-O 11. Bd2 Bxc3 12. bxc3 c5 13. e5 Nd5 14. c4 Nb4 15. Bxb4 cxb4 16. Qe4 Nc5 17. Qxb4 Rc8 18. Be2 Nd3 19. Qd2 Nxe5 20. Nxe5 Qxd4 21. Qxd4 Rxc4 22. Qd2 Ra4 23. Rxa4 Bxa4 24. Rd1 Rc8 25. h3 Rc2 26. Qd4 Rxe2 27. Qxa4 Rxf2 28. Qd7 Rf1+ 29. Rxf1 Bxf1 30. Qd8+ Kh7 31. Qd4 Ba6 32. Qa7 b5 33. axb5 Bxb5 0-1"""
        ]
        
        with open(output_path, 'w') as f:
            for game in sample_games:
                f.write(game.strip() + "\n\n")
        
        logger.info(f"Created minimal sample: {output_path}")


def main():
    parser = argparse.ArgumentParser(description="Collect chess training data")
    parser.add_argument("--action", choices=["download", "sample", "filter", "lichess"], 
                       default="sample", help="Action to perform")
    parser.add_argument("--year", type=int, default=2023, help="Year for Lichess data")
    parser.add_argument("--month", type=int, default=1, help="Month for Lichess data")
    parser.add_argument("--variant", default="standard", help="Game variant")
    parser.add_argument("--input", type=str, help="Input PGN file")
    parser.add_argument("--output", type=str, help="Output file")
    parser.add_argument("--max-games", type=int, default=10000, help="Max games for sample")
    parser.add_argument("--min-elo", type=int, default=1800, help="Minimum rating")
    parser.add_argument("--data-dir", default="data", help="Data directory")
    
    args = parser.parse_args()
    
    collector = DataCollector(args.data_dir)
    
    if args.action == "sample":
        # Create sample training data
        output_file = collector.download_sample_games()
        logger.info(f"Sample data ready: {output_file}")
        
    elif args.action == "lichess":
        # Download Lichess database
        pgn_file = collector.download_lichess_database(args.year, args.month, args.variant)
        if pgn_file:
            logger.info(f"Lichess data ready: {pgn_file}")
        
    elif args.action == "filter" and args.input and args.output:
        # Filter existing PGN file
        collector.filter_games_by_criteria(args.input, args.output, args.min_elo)
        
    elif args.action == "sample" and args.input and args.output:
        # Create sample from existing file
        collector.create_sample_dataset(args.input, args.output, args.max_games)
    
    else:
        logger.error("Invalid arguments. Use --help for usage information.")


if __name__ == "__main__":
    main()