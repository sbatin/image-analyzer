export default class API {
  static async analyze(path, params) {
    const resp = await fetch(`/analyze?path=${path}&dist=${params.distance}&hashType=${params.hashType}&hashSize=${params.hashSize}`, {
      method: 'POST',
    });
  
    return resp.json();
  }

  static async poll(taskId) {
    const resp = await fetch(`/poll?taskId=${taskId}`);
    return resp.json();
  }

  static subscribe(taskId, handler) {
    const evtSource = new EventSource(`/subscribe?taskId=${taskId}`);
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