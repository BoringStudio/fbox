import React from 'react';

type WsResponseContainer<T, C> = { type: T; content: C };
type WsResponse =
  | WsResponseContainer<'created', { phrase: string }>
  | WsResponseContainer<'connected', { seed: string }>
  | WsResponseContainer<'peer_not_found', undefined>;

type OnCreatedHandler = (phrase: string) => void;
type OnConnectedHandler = (seed: string) => void;
type OnPeerNotFoundHandler = () => void;

class SessionSocketBuilder {
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

  onDisconnect(handler: () => void): SessionSocketBuilder {
    this.onDisconnectedHandler = handler;
    return this;
  }

  build(): WebSocket {
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

    return socket;
  }
}

let wsSocket: WebSocket | null = null;

interface IProps {}

interface IState {
  phrase: string | null;
}

export class WebSocketService extends React.Component<IProps, IState> {
  private builder: SessionSocketBuilder;

  constructor(props: IProps) {
    super(props);

    this.builder = new SessionSocketBuilder().onCreate(this.onSocketCreated);

    this.state = {
      phrase: null
    };
  }

  componentDidMount() {
    console.log(wsSocket);
    wsSocket?.close();
    wsSocket = this.builder.build();
    console.log(wsSocket);
  }

  onSocketCreated = (phrase: string) => {
    console.log(phrase);
    this.setState({
      phrase
    });
  };

  render() {
    const { phrase } = this.state;

    return (
      <div>
        <pre>phrase: {phrase}</pre>
      </div>
    );
  }
}
