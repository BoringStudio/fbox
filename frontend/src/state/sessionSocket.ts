export type WsRequestKind = 'connect';

export type WsRequestBody<T extends WsRequestKind> =
  | (T extends 'connect' ? { phrase: string } : never)
  | never;

export type WsRequestContainer<K extends WsRequestKind> = {
  type: K;
  content: WsRequestBody<K>;
};
export type WsRequest = WsRequestContainer<'connect'>;

export type WsResponseContainer<T, C> = { type: T; content: C };
export type WsResponse =
  | WsResponseContainer<'created', { phrase: string }>
  | WsResponseContainer<'connected', { seed: string }>
  | WsResponseContainer<'peer_not_found', undefined>;

export type OnCreatedHandler = (phrase: string) => void;
export type OnConnectedHandler = (seed: string) => void;
export type OnPeerNotFoundHandler = () => void;

export class SessionSocket {
  private socket: WebSocket;

  constructor(socket: WebSocket) {
    this.socket = socket;
  }

  send<K extends WsRequestKind>(kind: K, content: WsRequestBody<K>) {
    this.socket.send(JSON.stringify({ type: kind, content } as WsRequest));
  }

  close() {
    this.socket.close();
  }
}

export class SessionSocketBuilder {
  private url: string;
  private onCreatedHandler: OnCreatedHandler | null = null;
  private onConnectedHandler: OnConnectedHandler | null = null;
  private onPeerNotFoundHandler: OnPeerNotFoundHandler | null = null;
  private onDisconnectedHandler: (() => void) | null = null;

  constructor(url?: string) {
    this.url = url || process.env.REACT_APP_SESSIONS_SOCKET;
  }

  withUrl(url: string): SessionSocketBuilder {
    this.url = url;
    return this;
  }

  onCreate(handler: OnCreatedHandler): SessionSocketBuilder {
    this.onCreatedHandler = handler;
    return this;
  }

  onConnect(handler: OnConnectedHandler): SessionSocketBuilder {
    this.onConnectedHandler = handler;
    return this;
  }

  onPeerNotFound(handler: OnPeerNotFoundHandler): SessionSocketBuilder {
    this.onPeerNotFoundHandler = handler;
    return this;
  }

  onDisconnect(handler: () => void): SessionSocketBuilder {
    this.onDisconnectedHandler = handler;
    return this;
  }

  build(): SessionSocket {
    const socket = new WebSocket(this.url);
    socket.onmessage = message => {
      const data: WsResponse = JSON.parse(message.data);
      console.log(message);

      switch (data.type) {
        case 'created':
          return this.onCreatedHandler?.(data.content.phrase);
        case 'connected':
          return this.onConnectedHandler?.(data.content.seed);
        case 'peer_not_found':
          return this.onPeerNotFoundHandler?.();
        default:
          console.warn('got unknown response from server:', data);
          break;
      }
    };
    this.onDisconnectedHandler && (socket.onclose = this.onDisconnectedHandler);

    return new SessionSocket(socket);
  }
}
