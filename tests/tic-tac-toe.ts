import * as anchor from '@coral-xyz/anchor';
import {AnchorError, Program, Wallet} from '@coral-xyz/anchor';
import {TicTacToe} from '../target/types/tic_tac_toe';
import chai, {expect} from 'chai';
import chaiAsPromised from 'chai-as-promised';

type Keypair = anchor.web3.Keypair;
type PublicKey = anchor.web3.PublicKey;

type Tile = [number, number]; // (x, y) coordinates for a play

type GameState =
    | { active: {} }
    | { tie: {} }
    | { won: { winner: PublicKey } };

const ACTIVE_STATE: GameState = { active: {} };
const TIE_STATE: GameState = { tie: {} };


type Sign = { x: {} } | { o: {} } | null;

type Board = [
    [Sign, Sign, Sign],
    [Sign, Sign, Sign],
    [Sign, Sign, Sign]
];

chai.use(chaiAsPromised);

function addressString(publicKey: PublicKey, name: string) {
    const address = publicKey.toString();
    const shortenedAddress = `${address.slice(0, 4)}...${address.slice(-4)}`;
    return name ? `${shortenedAddress} (${name})` : shortenedAddress;
}

class Game {
    public readonly program: Program<TicTacToe>;
    private readonly programProvider: anchor.AnchorProvider;
    public gameKeypair:  anchor.web3.Keypair;
    public playerOne:  Wallet;
    public playerTwo:  anchor.web3.Keypair;
    public turnNumber: number;
    public expectedBoard: Board;

    constructor(program: Program<TicTacToe>) {
        this.program = program;
        this.programProvider = program.provider as anchor.AnchorProvider;
    }

    public async printBalance(name: string, publicKey: anchor.web3.PublicKey) {
        const balance: number = await this.program.provider.connection.getBalance(publicKey);
        const printedAddr = addressString(publicKey, name);
        console.log(`Balance of ${printedAddr}: ${balance} lamports`);
    }

    public playerOnePubkey(): PublicKey {
        return this.playerOne.publicKey;
    }

    public async setupGame(playerOne: Wallet | null = null, playerTwo: Keypair | null = null) {
        if (playerOne === null) {
            playerOne = this.programProvider.wallet as Wallet;
        }
        this.playerOne = playerOne;

        if (playerTwo === null) {
            playerTwo = anchor.web3.Keypair.generate();
        }
        this.playerTwo = playerTwo;

        this.turnNumber = 1;
        this.gameKeypair = anchor.web3.Keypair.generate();

        //await this.printBalance("game key at start", this.gameKeypair.publicKey);
        //await this.printBalance("player one at start", this.playerOne.publicKey);
        //await this.printBalance("player two at start", this.playerTwo.publicKey);

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

        //await this.printBalance("game after setup", this.gameKeypair.publicKey);
        //await this.printBalance("player one after setup", this.playerOne.publicKey);
        //await this.printBalance("player two after setup", this.playerTwo.publicKey);
    }

    public async play(tile: Tile, expectedGameState: GameState): Promise<void> {

        // let variable named "player" be equal to player1 if the turn is odd, else player2
        let player:Keypair|Wallet = this.turnNumber % 2 === 1 ? this.playerOne : this.playerTwo;

        //await this.printBalance("before play", player.publicKey);

        let expectedBoard: Board = [...this.expectedBoard];
        let expectedTurn = this.turnNumber;
        if (expectedGameState.hasOwnProperty("active")) {
            expectedTurn += 1;
        }

        const [row, col] = tile;

        if (row >= 0 && row <= 2 && col >= 0 && col <= 2 && expectedBoard[row][col] === null) {
            expectedBoard[row][col] = ((this.turnNumber % 2 === 1) ? {x: {}} : {o: {}});
        }

        await this.program.methods
            .play({row, column: col})
            .accounts({
                player: player.publicKey,
                game: this.gameKeypair.publicKey,
            })
            .signers(player instanceof (anchor.Wallet as any) ? [] : [player])
            .rpc();

        const gameState = await this.program.account.game.fetch(this.gameKeypair.publicKey);
        expect(gameState.turn).to.equal(expectedTurn);
        expect(gameState.state).to.eql(expectedGameState);
        expect(gameState.board)
            .to
            .eql(expectedBoard);

        //await this.printBalance("after play", player.publicKey);

        // play() above didn't error, so we can update the turn number
        this.turnNumber = expectedTurn;
    }
}

describe('tic-tac-toe', () => {
    // Configure the client to use the local cluster.
    anchor.setProvider(anchor.AnchorProvider.env());

    const program = anchor.workspace.TicTacToe as Program<TicTacToe>;

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

        // try {
        //     await game.play(
        //         program,
        //         gameKeypair.publicKey,
        //         playerOne, // same player in subsequent turns
        //         // change sth about the tx because
        //         // duplicate tx that come in too fast
        //         // after each other may get dropped
        //         [1, 0],
        //         2,
        //         {active: {},},
        //         [
        //             [{x: {}}, null, null],
        //             [null, null, null],
        //             [null, null, null]
        //         ]
        //     );
        //     chai.assert(false, "should've failed but didn't ");
        // } catch (_err) {
        //     expect(_err).to.be.instanceOf(AnchorError);
        //     const err: AnchorError = _err;
        //     expect(err.error.errorCode.code).to.equal("NotPlayersTurn");
        //     expect(err.error.errorCode.number).to.equal(6003);
        //     expect(err.program.equals(program.programId)).is.true;
        //     expect(err.error.comparedValues).to.deep.equal([playerTwo.publicKey, playerOne.publicKey]);
        // }

        await game.play([1, 0], ACTIVE_STATE);
        await game.play([0, 1], ACTIVE_STATE);

        try {
            // row 5 is out of bounds
            await game.play([5, 1], ACTIVE_STATE);
            chai.assert(false, "should've failed but didn't ");
        } catch (_err) {
            expect(_err).to.be.instanceOf(AnchorError);
            const err: AnchorError = _err;
            expect(err.error.errorCode.number).to.equal(6000);
            expect(err.error.errorCode.code).to.equal("TileOutOfBounds");
        }

        await game.play([1, 1], ACTIVE_STATE);

        try {
            await game.play([0, 0], ACTIVE_STATE);
            chai.assert(false, "should've failed but didn't ");
        } catch (_err) {
            expect(_err).to.be.instanceOf(AnchorError);
            const err: AnchorError = _err;
            expect(err.error.errorCode.number).to.equal(6001);
            expect(err.error.errorCode.code).to.equal("TileAlreadySet");
        }

        await game.play([0, 2], {won: {winner: game.playerOnePubkey()}});

        try {
            // make a play after the game is won
            await game.play([0, 2], {won: {winner: game.playerOnePubkey()}});
            chai.assert(false, "should've failed but didn't ");
        } catch (_err) {
            expect(_err).to.be.instanceOf(AnchorError);
            const err: AnchorError = _err;
            expect(err.error.errorCode.number).to.equal(6002);
            expect(err.error.errorCode.code).to.equal("GameAlreadyOver");
        }
    })

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
});
