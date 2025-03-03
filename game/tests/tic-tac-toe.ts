import * as anchor from '@coral-xyz/anchor';
import {Wallet} from '@coral-xyz/anchor';
import {TicTacToe} from '../target/types/tic_tac_toe';
import chai, {expect} from 'chai';
import chaiAsPromised from 'chai-as-promised';

chai.use(chaiAsPromised);

type Tile = [number, number]; // (x, y) coordinates for a play

type GameState =
    | { active: {} }
    | { tie: {} }
    | { won: { winner: anchor.web3.PublicKey } };

const ACTIVE_STATE: GameState = {active: {}};
const TIE_STATE: GameState = {tie: {}};

type Sign = { x: {} } | { o: {} } | null;

type Board = [
    [Sign, Sign, Sign],
    [Sign, Sign, Sign],
    [Sign, Sign, Sign]
];

function addressString(publicKey: anchor.web3.PublicKey, name: string) {
    const address = publicKey.toString();
    const shortenedAddress = `${address.slice(0, 4)}...${address.slice(-4)}`;
    return name ? `${shortenedAddress} (${name})` : shortenedAddress;
}

class Game {
    public readonly program: anchor.Program<TicTacToe>;
    public readonly programProvider: anchor.AnchorProvider;
    private readonly printBalances: boolean = true;

    public gameKeypair: anchor.web3.Keypair;
    public playerOne: Wallet;
    public playerTwo: anchor.web3.Keypair;
    private turnNumber: number;
    private expectedBoard: Board;

    constructor(program: anchor.Program<TicTacToe>) {
        this.program = program;
        this.programProvider = program.provider as anchor.AnchorProvider;
    }

    public async printBalance(name: string, publicKey: anchor.web3.PublicKey) {
        const balance: number = await this.program.provider.connection.getBalance(publicKey);
        const printedAddr = addressString(publicKey, name);
        console.log(`Balance of ${printedAddr}: ${balance} lamports`);
    }

    public async setupGame(
        playerOne: anchor.Wallet | null = null,
        playerTwo: anchor.web3.Keypair | null = null,
    ) {
        if (playerOne === null) {
            playerOne = this.programProvider.wallet as anchor.Wallet;
        }
        this.playerOne = playerOne;

        if (playerTwo === null) {
            playerTwo = anchor.web3.Keypair.generate();
        }
        this.playerTwo = playerTwo;

        this.turnNumber = 1;
        this.gameKeypair = anchor.web3.Keypair.generate();

        if (this.printBalances) {
            await this.printBalance("game key at start", this.gameKeypair.publicKey);
            await this.printBalance("player one at start", this.playerOne.publicKey);
            await this.printBalance("player two at start", this.playerTwo.publicKey);
        }

        await this.program.methods
            .setupGame(playerTwo.publicKey)
            .accounts({
                game: this.gameKeypair.publicKey,
                playerOne: this.playerOne.publicKey,
            })
            .signers([this.gameKeypair])
            .rpc();

        this.expectedBoard = [[null, null, null], [null, null, null], [null, null, null]];

        let gameState = await this.program.account.game.fetch(this.gameKeypair.publicKey);
        expect(gameState.turn).to.equal(1);
        expect(gameState.players).to.eql([playerOne.publicKey, playerTwo.publicKey]);
        expect(gameState.state).to.eql(ACTIVE_STATE);
        expect(gameState.board).to.eql(this.expectedBoard);

        if (this.printBalances) {
            await this.printBalance("game after setup", this.gameKeypair.publicKey);
            await this.printBalance("player one after setup", this.playerOne.publicKey);
            await this.printBalance("player two after setup", this.playerTwo.publicKey);
        }
    }

    public async play(tile: Tile, expectedGameState: GameState): Promise<void> {

        let isPlayerOne = this.turnNumber % 2 === 1;
        let playerPubKey = isPlayerOne ? this.playerOne.publicKey : this.playerTwo.publicKey;
        let signers: anchor.web3.Signer[] = isPlayerOne ? [] : [this.playerTwo];

        if (this.printBalances) {
            await this.printBalance("before play", playerPubKey);
        }

        let expectedBoard: Board = [...this.expectedBoard];
        let expectedTurn = this.turnNumber;
        if (expectedGameState.hasOwnProperty("active")) {
            expectedTurn += 1;
        }

        const [row, col] = tile;

        // Callers can invoke error cases, in which case we want the error to come from the
        // contract and no the test.
        if (row >= 0 && row <= 2 && col >= 0 && col <= 2 && expectedBoard[row][col] === null) {
            expectedBoard[row][col] = isPlayerOne ? {x: {}} : {o: {}};
        }

        await this.program.methods
            .play([row, col])
            .accounts({
                player: playerPubKey,
                game: this.gameKeypair.publicKey,
            })
            .signers(signers)
            .rpc();

        const gameState = await this.program.account.game.fetch(this.gameKeypair.publicKey);
        expect(gameState.turn).to.equal(expectedTurn);
        expect(gameState.state).to.eql(expectedGameState);
        expect(gameState.board)
            .to
            .eql(expectedBoard);

        if (this.printBalances) {
            await this.printBalance("after play", playerPubKey);
        }

        // play() above didn't error, so we can update the turn number
        this.turnNumber = expectedTurn;
    }
}

describe('tic-tac-toe', () => {
    // Configure the client to use the local cluster.
    anchor.setProvider(anchor.AnchorProvider.env());

    const program = anchor.workspace.TicTacToe as anchor.Program<TicTacToe>;

    it('setup game!', async () => {
        console.log("setup game test starting");
        let game = new Game(program);
        await game.setupGame(null, null);
    });

    it('player one wins!', async () => {
        console.log("player one wins test starting");

        let game = new Game(program);
        await game.setupGame(null, null);

        await game.play([0, 0], ACTIVE_STATE);
        await game.play([1, 0], ACTIVE_STATE);
        await game.play([0, 1], ACTIVE_STATE);
        await game.play([1, 1], ACTIVE_STATE);
        await game.play([0, 2], {won: {winner: game.playerOne.publicKey}});
    })

    it('player two wins!', async () => {
        console.log("player two wins test starting");
        let game = new Game(program);
        await game.setupGame(null, null);

        // player 2 takes the diagonal
        await game.play([0, 1], ACTIVE_STATE);
        await game.play([0, 0], ACTIVE_STATE);
        await game.play([1, 0], ACTIVE_STATE);
        await game.play([1, 1], ACTIVE_STATE);
        await game.play([2, 1], ACTIVE_STATE);
        await game.play([2, 2], {won: {winner: game.playerTwo.publicKey}});
    });

    it('tie', async () => {
        console.log("tie test starting");
        let game = new Game(program);
        await game.setupGame(null, null);

        await game.play([0, 0], ACTIVE_STATE);
        await game.play([1, 1], ACTIVE_STATE);
        await game.play([2, 0], ACTIVE_STATE);
        await game.play([1, 0], ACTIVE_STATE);
        await game.play([1, 2], ACTIVE_STATE);
        await game.play([0, 1], ACTIVE_STATE);
        await game.play([2, 1], ACTIVE_STATE);
        await game.play([2, 2], ACTIVE_STATE);
        await game.play([0, 2], TIE_STATE);
    })

    it('not player\'s turn', async () => {
        console.log("not player\'s turn test starting");
        let game = new Game(program);
        let playerOne = game.programProvider.wallet as Wallet;
        let playerTwo = anchor.web3.Keypair.generate();
        await game.setupGame(playerOne, playerTwo);

        // Have player2 go first
        try {
            await game.program.methods
                .play([0, 0])
                .accounts({
                    player: playerTwo.publicKey,
                    game: game.gameKeypair.publicKey,
                })
                .signers([playerTwo])
                .rpc();
        } catch (_err) {
            expect(_err).to.be.instanceOf(anchor.AnchorError);
            const err: anchor.AnchorError = _err;
            expect(err.error.errorCode.number).to.equal(6003);
            expect(err.error.errorCode.code).to.equal("NotPlayersTurn");
        }

        // let playerOne have his turn
        await game.play([0, 0], ACTIVE_STATE);

        // Now have playerOne move out of turn
        try {
            await game.program.methods
                .play([1, 0])
                .accounts({
                    player: playerOne.publicKey,
                    game: game.gameKeypair.publicKey,
                })
                .signers([])
                .rpc();
        } catch (_err) {
            expect(_err).to.be.instanceOf(anchor.AnchorError);
            const err: anchor.AnchorError = _err;
            expect(err.error.errorCode.number).to.equal(6003);
            expect(err.error.errorCode.code).to.equal("NotPlayersTurn");
        }
    })

    it('out of bounds play', async () => {
        console.log("out of bounds play test starting");

        let game = new Game(program);
        await game.setupGame(null, null);

        // The tile values are represented by u8 in Rust. If we try to add
        // negative values, we'll get a range error from Node, not the contract.
        let outOfBoundsPairs: Tile[] = [
            [3, 0],
            [0, 3],
            [3, 3],
            [0, 5],
            [5, 0],
        ];

        for (let tile of outOfBoundsPairs) {
            try {
                await game.play(tile, ACTIVE_STATE);
                chai.assert(false, "should've failed but didn't");
            } catch (_err) {
                expect(_err).to.be.instanceOf(anchor.AnchorError);
                const err: anchor.AnchorError = _err;
                expect(err.error.errorCode.number).to.equal(6000);
                expect(err.error.errorCode.code).to.equal("TileOutOfBounds");
            }
        }
    });

    it('tile already set', async () => {
        console.log("tile already set test starting");

        let game = new Game(program);
        await game.setupGame(null, null);

        await game.play([0, 0], ACTIVE_STATE);

        try {
            await game.play([0, 0], ACTIVE_STATE);
            chai.assert(false, "should've failed but didn't");
        } catch (_err) {
            expect(_err).to.be.instanceOf(anchor.AnchorError);
            const err: anchor.AnchorError = _err;
            expect(err.error.errorCode.number).to.equal(6001);
            expect(err.error.errorCode.code).to.equal("TileAlreadySet");
        }
    });

    it('game already started', async () => {
        console.log("game already started test starting");

        let game = new Game(program);
        await game.setupGame(null, null);

        try {
            // Second call fails
            await game.program.methods
                .setupGame(game.playerTwo.publicKey)
                .accounts({
                    game: game.gameKeypair.publicKey,
                    playerOne: game.playerOne.publicKey,
                })
                .signers([game.gameKeypair])
                .rpc();
            chai.assert(false, "should've failed but didn't");
        } catch (_err) {
            // I'm not able to trigger the `GameAlreadyStarted` error. A different
            // error that the account address is already in use is thrown first.
            let errStr = JSON.stringify(_err);
            let expected =
                `"Allocate: account Address { address: ${game.gameKeypair.publicKey}, base: None } already in use"`
            expect(errStr).to.contain(expected);
        }
    });

    it('game already over!', async () => {
        console.log("game already over test starting");

        let game = new Game(program);
        await game.setupGame(null, null);

        await game.play([2, 2], ACTIVE_STATE);
        await game.play([0, 1], ACTIVE_STATE);
        await game.play([1, 1], ACTIVE_STATE);
        await game.play([0, 2], ACTIVE_STATE);
        await game.play([0, 0], {won: {winner: game.playerOne.publicKey}});

        // Make a play after the game is already won
        try {
            await game.play([2, 0], ACTIVE_STATE);
            chai.assert(false, "should've failed but didn't ");
        } catch (_err) {
            expect(_err).to.be.instanceOf(anchor.AnchorError);
            const err: anchor.AnchorError = _err;
            expect(err.error.errorCode.number).to.equal(6002);
            expect(err.error.errorCode.code).to.equal("GameAlreadyOver");
        }
    })
});
