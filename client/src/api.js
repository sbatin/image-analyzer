export default class API {
  static async analyze(path, distance) {
    const resp = await fetch(`/analyze?path=${path}&dist=${distance}`, {
      method: 'POST',
    });
  
    return resp.json();
  }

  static async poll(path) {
    const resp = await fetch(`/poll?path=${path}`);
    return resp.json();
  }

  static subscribe(path, handler) {
    const evtSource = new EventSource(`/subscribe?path=${path}`);
    evtSource.onmessage = (event) => {
      handler(JSON.parse(event.data));
    };
    evtSource.onerror = () => {
      evtSource.close();
    };
  }

  static async listDir(path) {
    const resp = await fetch(`/list_folder?path=${path}`);
    return resp.json();
  }
}