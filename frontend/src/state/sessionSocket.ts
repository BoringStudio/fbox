// session socket

export class SessionSocket {
  private socket: WebSocket;

  constructor(socket: WebSocket) {
    this.socket = socket;
  }

  send<K extends WsRequestType>(kind: K, content: WsRequestContent[K]) {
    this.socket.send(JSON.stringify({ type: kind, content } as WsRequest));
  }

  close() {
    this.socket.close();
  }
}

// session socket builder

export class SessionSocketBuilder {
  private url: string;

  public responseHandlers: { [T in WsResponseType]?: WsResponseHandler<T> } = {};
  public onDisconnected: (() => void) | null = null;

  constructor(url?: string) {
    this.url = url || process.env.REACT_APP_SESSIONS_SOCKET;
  }

  withUrl(url: string): SessionSocketBuilder {
    this.url = url;
    return this;
  }

  build(): SessionSocket {
    const socket = new WebSocket(this.url);
    socket.onmessage = message => {
      const data: WsResponse = JSON.parse(message.data);
      console.log(message);

      (this.responseHandlers[data.type] as any)?.(data.content);
    };
    this.onDisconnected && (socket.onclose = this.onDisconnected);

    return new SessionSocket(socket);
  }
}

// requests

export type WsRequestContent = {
  connect: { phrase: string };
  add_file: { name: string; size: number };
  remove_file: { name: string };
};
export type WsRequestType = keyof WsRequestContent;
export type WsRequestContainer<T extends WsRequestType> = { type: T; content: WsRequestContent[T] } | never;
export type WsRequest = WsRequestContainer<WsRequestType>;

// responses

export type WsResponseContent = {
  created: { phrase: string };
  connected: { seed: string; files: [FileInfo] };
  peer_not_found: null;
  session_not_found: null;
  file_count_limit_reached: null;
  file_added: { id: string; name: string; size: number };
  file_removed: { id: string };
};
export type WsResponseType = keyof WsResponseContent;
export type WsResponseContainer<T extends WsResponseType> = { type: T; content: WsResponseContent[T] } | never;
export type WsResponse = WsResponseContainer<WsResponseType>;

// stuff

export type FileInfo = {
  id: string;
  name: string;
  size: number;
};

// handlers

export type WsResponseHandler<T extends WsResponseType> = ((content: WsResponseContent[T]) => void) | never;
