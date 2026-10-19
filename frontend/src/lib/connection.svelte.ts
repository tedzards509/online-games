import type {
  Action,
  ClientMessage,
  Membership,
  ServerMessage,
  Snapshot,
  Schemas,
} from './types';

export class RoomConnection {
  room = $state<Snapshot | null>(null);
  status = $state<'connecting' | 'connected' | 'offline'>('connecting');
  error = $state('');
  pending = $state(false);
  private socket: WebSocket | null = null;
  private timer: ReturnType<typeof setTimeout> | undefined;
  private disposed = false;
  private retry = 500;

  constructor(private member: Membership) {
    this.connect();
  }

  private connect(): void {
    if (this.disposed) return;
    this.status = 'connecting';
    const url = new URL(`/api/rooms/${this.member.code}/ws`, location.href);
    url.protocol = location.protocol === 'https:' ? 'wss:' : 'ws:';
    const socket = new WebSocket(url);
    this.socket = socket;
    socket.onopen = () =>
      socket.send(
        JSON.stringify({
          type: 'authenticate',
          token: this.member.token,
        } satisfies ClientMessage),
      );
    socket.onmessage = (event) => {
      const message: ServerMessage = JSON.parse(event.data);
      if (message.type === 'snapshot') {
        if (message.acknowledged) this.pending = false;
        this.room = message.room;
        this.status = 'connected';
        this.retry = 500;
      } else {
        this.pending = false;
        this.error = message.error;
        if (message.error === 'Invalid room token') this.close();
      }
    };
    socket.onclose = () => {
      this.status = 'offline';
      this.pending = false;
      if (!this.disposed) {
        this.timer = setTimeout(() => this.connect(), this.retry);
        this.retry = Math.min(this.retry * 2, 10000);
      }
    };
    socket.onerror = () => socket.close();
  }

  private send(message: ClientMessage): void {
    if (
      this.status !== 'connected' ||
      this.pending ||
      this.socket?.readyState !== WebSocket.OPEN
    )
      return;
    this.error = '';
    this.pending = true;
    this.socket.send(JSON.stringify(message));
  }

  move(action: Action): void {
    this.gameMessage({ type: 'move', action });
  }

  start(): void {
    if (this.room) this.send({ type: 'start', revision: this.room.revision });
  }

  finish(won: boolean): void {
    this.gameMessage({ type: 'finish', won });
  }

  private gameMessage(message: Schemas['GameMessage']): void {
    if (this.room?.game)
      this.send({
        type: 'game',
        revision: this.room.revision,
        game_id: this.room.game.id,
        message,
      });
  }

  configure(config: Schemas['Config'] | null): void {
    if (this.room)
      this.send({ type: 'configure', revision: this.room.revision, config });
  }

  seat(members: number[]): void {
    if (this.room)
      this.send({ type: 'seat', revision: this.room.revision, members });
  }

  abort(): void {
    if (this.room?.game)
      this.send({
        type: 'abort',
        revision: this.room.revision,
        game_id: this.room.game.id,
      });
  }

  close(): void {
    this.disposed = true;
    clearTimeout(this.timer);
    this.socket?.close();
    this.status = 'offline';
  }
}
