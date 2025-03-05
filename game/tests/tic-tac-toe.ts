import * as anchor from '@coral-xyz/anchor';
import { AnchorProvider, Provider, Wallet } from '@coral-xyz/anchor';
import type { TicTacToe } from '../target/types/tic_tac_toe';
import chai, { expect } from 'chai';
import chaiAsPromised from 'chai-as-promised';

chai.use(chaiAsPromised);

type Tile = [number, number]; // (x, y) coordinates for a play

type GameState = { active: object } | { tie: object } | { won: { winner: anchor.web3.PublicKey } };

const ACTIVE_STATE: GameState = { active: {} };
const TIE_STATE: GameState = { tie: {} };

type Sign = { x: object } | { o: object } | null;

// prettier-ignore
type Board = [
  [Sign, Sign, Sign],
  [Sign, Sign, Sign],
  [Sign, Sign, Sign]
];

type GameAccount = {
  players: [anchor.web3.PublicKey, anchor.web3.PublicKey];
  turn: number;
  state: GameState;
  board: Board;
};

class Player {
  public readonly program: anchor.Program<TicTacToe>;
  private readonly printBalances: boolean = false;

  // set by setupGame or joinGame
  //public opponent: anchor.web3.PublicKey = null; // Public key of the opponent
  public gameID: anchor.web3.PublicKey = null; // Public key of the Game data account

  constructor(program: anchor.Program<TicTacToe>, gameID: anchor.web3.PublicKey) {
    this.program = program;
    this.gameID = gameID;
  }

  public pubkey() {
    return this.program.provider.publicKey;
  }

  private async gameAccount(): Promise<GameAccount> {
    return (await this.program.account.game.fetch(this.gameID)) as GameAccount;
  }

  public async printBalance(prefix: string) {
    if (!this.printBalances) {
      return;
    }
    const balance: number = await this.program.provider.connection.getBalance(this.pubkey());
    prefix = prefix ? `${prefix}: ` : '';
    const address = this.pubkey().toString();
    console.log(
      `${prefix}Balance of ${address.slice(0, 4)}...${address.slice(-4)}: ${balance / anchor.web3.LAMPORTS_PER_SOL} SOL`,
    );
  }

  public async setupGame(gameKeypair: anchor.web3.Keypair, opponent: anchor.web3.PublicKey) {
    // The private key is only needed by this function, but the public key
    // was saved during construction.
    expect(this.gameID).eql(gameKeypair.publicKey);

    await this.printBalance('player one before setupGame');

    await this.program.methods
      .setupGame(opponent)
      .accounts({
        game: this.gameID,
        playerOne: this.pubkey(),
      })
      .signers([gameKeypair])
      .rpc();

    const gameState = await this.gameAccount();
    expect(gameState.turn).to.equal(1);
    expect(gameState.players).to.eql([this.pubkey(), opponent]);
    expect(gameState.state).to.eql(ACTIVE_STATE);
    expect(gameState.board).to.eql([
      [null, null, null],
      [null, null, null],
      [null, null, null],
    ]);

    await this.printBalance('player one after setupGame');
  }

  public async play(tile: Tile, expectedState: GameState): Promise<void> {
    await this.printBalance('before play');

    const gameBefore = await this.gameAccount();

    const [row, col] = tile;

    await this.program.methods
      .play([row, col])
      .accounts({
        player: this.pubkey(),
        game: this.gameID,
      })
      .signers([]) // TODO: Can I just remove this?
      .rpc();

    const expectedBoard = gameBefore.board;
    if (row >= 0 && row <= 2 && col >= 0 && col <= 2 && expectedBoard[row][col] === null) {
      expectedBoard[row][col] = gameBefore.turn % 2 === 1 ? { x: {} } : { o: {} };
    }

    const expectedTurn: number =
      expectedState === ACTIVE_STATE ? gameBefore.turn + 1 : gameBefore.turn;

    // Update the gameAccount after the play
    const gameAfter = await this.gameAccount();
    expect(expectedTurn).to.equal(gameAfter.turn);
    expect(expectedBoard).to.eql(gameAfter.board);
    expect(expectedState).to.eql(gameAfter.state);

    await this.printBalance('after play');
  }
}

async function transfer(
  provider: Provider,
  to: anchor.web3.PublicKey,
  amount: number,
): Promise<anchor.web3.TransactionSignature> {
  return await provider.sendAndConfirm(
    new anchor.web3.Transaction().add(
      anchor.web3.SystemProgram.transfer({
        fromPubkey: provider.publicKey,
        toPubkey: to,
        lamports: amount, // Amount in lamports
      }),
    ),
  );
}

describe('tic-tac-toe', function () {
  let programOne: anchor.Program<TicTacToe>;
  let programTwo: anchor.Program<TicTacToe>;

  // Give player two 1/10th of a SOL for transaction fees
  before(async function () {
    anchor.setProvider(anchor.AnchorProvider.env());
    programOne = anchor.workspace.TicTacToe as anchor.Program<TicTacToe>;

    // Create a new provider that is the same as the default provider,
    // but with a different wallet for player two.
    const playerTwoProvider = new AnchorProvider(
      anchor.getProvider().connection,
      new Wallet(anchor.web3.Keypair.generate()),
      AnchorProvider.defaultOptions(),
    );

    // Because anchor embeds a wallet directly into the program object, we need a
    // separate program for player two.
    programTwo = new anchor.Program(
      programOne.idl,
      playerTwoProvider,
      programOne.coder,
    ) as anchor.Program<TicTacToe>;

    // Fund player two
    await transfer(
      programOne.provider,
      playerTwoProvider.publicKey,
      anchor.web3.LAMPORTS_PER_SOL / 10,
    );
  });

  async function startNewGame(
    gameKeyPair: anchor.web3.Keypair = anchor.web3.Keypair.generate(),
  ): Promise<{ playerOne: Player; playerTwo: Player }> {
    const playerOne = new Player(programOne, gameKeyPair.publicKey);
    const playerTwo = new Player(programTwo, gameKeyPair.publicKey);
    await playerOne.setupGame(gameKeyPair, playerTwo.pubkey());
    return { playerOne, playerTwo };
  }

  it('setup game!', async function () {
    console.log('setup game test starting');
    const gameKeyPair = anchor.web3.Keypair.generate();
    const player = new Player(programOne, gameKeyPair.publicKey);
    const opponent = anchor.web3.Keypair.generate().publicKey;
    await player.setupGame(gameKeyPair, opponent);
  });

  it('player one wins!', async function () {
    console.log('player one wins test starting');
    const { playerOne, playerTwo } = await startNewGame();
    await playerOne.play([0, 0], ACTIVE_STATE);
    await playerTwo.play([1, 0], ACTIVE_STATE);
    await playerOne.play([0, 1], ACTIVE_STATE);
    await playerTwo.play([1, 1], ACTIVE_STATE);
    await playerOne.play([0, 2], { won: { winner: playerOne.pubkey() } });
  });

  it('player two wins!', async function () {
    console.log('player two wins test starting');
    const { playerOne, playerTwo } = await startNewGame();

    // player 2 takes the diagonal
    await playerOne.play([0, 1], ACTIVE_STATE);
    await playerTwo.play([0, 0], ACTIVE_STATE);
    await playerOne.play([1, 0], ACTIVE_STATE);
    await playerTwo.play([1, 1], ACTIVE_STATE);
    await playerOne.play([2, 1], ACTIVE_STATE);
    await playerTwo.play([2, 2], { won: { winner: playerTwo.pubkey() } });
  });

  it('tie', async function () {
    console.log('tie test starting');
    const { playerOne, playerTwo } = await startNewGame();

    await playerOne.play([0, 0], ACTIVE_STATE);
    await playerTwo.play([1, 1], ACTIVE_STATE);
    await playerOne.play([2, 0], ACTIVE_STATE);
    await playerTwo.play([1, 0], ACTIVE_STATE);
    await playerOne.play([1, 2], ACTIVE_STATE);
    await playerTwo.play([0, 1], ACTIVE_STATE);
    await playerOne.play([2, 1], ACTIVE_STATE);
    await playerTwo.play([2, 2], ACTIVE_STATE);
    await playerOne.play([0, 2], TIE_STATE);
  });

  it("not player's turn", async function () {
    console.log("not player's turn test starting");
    const { playerOne, playerTwo } = await startNewGame();

    // Have player2 go first
    try {
      await playerTwo.play([0, 0], ACTIVE_STATE);
    } catch (_err) {
      expect(_err).to.be.instanceOf(anchor.AnchorError);
      const err: anchor.AnchorError = _err;
      expect(err.error.errorCode.number).to.equal(6003);
      expect(err.error.errorCode.code).to.equal('NotPlayersTurn');
    }

    // let playerOne have his turn
    await playerOne.play([0, 0], ACTIVE_STATE);

    // Now have playerOne move out of turn
    try {
      await playerOne.play([1, 1], ACTIVE_STATE);
    } catch (_err) {
      expect(_err).to.be.instanceOf(anchor.AnchorError);
      const err: anchor.AnchorError = _err;
      expect(err.error.errorCode.code).to.equal('NotPlayersTurn');
      expect(err.error.errorCode.number).to.equal(6003);
    }
  });

  it('out of bounds play', async function () {
    console.log('out of bounds play test starting');
    const { playerOne } = await startNewGame();

    // The tile values are represented by u8 in Rust. If we try to add
    // negative values, we'll get a range error from Node, not the contract.
    const outOfBoundsPairs: Tile[] = [
      [3, 0],
      [0, 3],
      [3, 3],
      [0, 5],
      [5, 0],
    ];

    for (const tile of outOfBoundsPairs) {
      try {
        await playerOne.play(tile, ACTIVE_STATE);
        chai.assert(false, "should've failed but didn't");
      } catch (_err) {
        expect(_err).to.be.instanceOf(anchor.AnchorError);
        const err: anchor.AnchorError = _err;
        expect(err.error.errorCode.code).to.equal('TileOutOfBounds');
        expect(err.error.errorCode.number).to.equal(6000);
      }
    }
  });

  it('tile already set', async function () {
    console.log('tile already set test starting');
    const { playerOne, playerTwo } = await startNewGame();

    const tile: Tile = [1, 1];
    await playerOne.play(tile, ACTIVE_STATE);

    try {
      await playerTwo.play(tile, ACTIVE_STATE);
      chai.assert(false, "should've failed but didn't");
    } catch (_err) {
      expect(_err).to.be.instanceOf(anchor.AnchorError);
      const err: anchor.AnchorError = _err;
      expect(err.error.errorCode.code).to.equal('TileAlreadySet');
      expect(err.error.errorCode.number).to.equal(6001);
    }
  });

  it('game already started', async function () {
    console.log('game already started test starting');
    const gameKeyPair = anchor.web3.Keypair.generate();
    const { playerOne, playerTwo } = await startNewGame(gameKeyPair);

    for (const player of [playerOne, playerTwo]) {
      try {
        await player.setupGame(gameKeyPair, playerTwo.pubkey());
        chai.assert(false, "should've failed but didn't");
      } catch (_err) {
        // We can't trigger `GameAlreadyStarted` as the anchor code will throw
        // its own error before our setup_game code is called if the game account
        // already exists.
        const errStr = JSON.stringify(_err);
        const expected = `"Allocate: account Address { address: ${gameKeyPair.publicKey}, base: None } already in use"`;
        expect(errStr).to.contain(expected);
      }
    }
  });

  it('game already over!', async function () {
    console.log('game already over test starting');
    const { playerOne, playerTwo } = await startNewGame();

    await playerOne.play([2, 2], ACTIVE_STATE);
    await playerTwo.play([0, 1], ACTIVE_STATE);
    await playerOne.play([1, 1], ACTIVE_STATE);
    await playerTwo.play([0, 2], ACTIVE_STATE);
    await playerOne.play([0, 0], { won: { winner: playerOne.pubkey() } });

    // Make a play after the game is already won
    try {
      await playerTwo.play([2, 0], ACTIVE_STATE);
      chai.assert(false, "should've failed but didn't ");
    } catch (_err) {
      expect(_err).to.be.instanceOf(anchor.AnchorError);
      const err: anchor.AnchorError = _err;
      expect(err.error.errorCode.code).to.equal('GameAlreadyOver');
      expect(err.error.errorCode.number).to.equal(6002);
    }
  });
});
