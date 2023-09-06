export default class API {
  static async analyze(path, distance) {
    const resp = await fetch(`/analyze?path=${path}&dist=${distance}`, {
      method: 'POST',
    });
  
    return resp.json();
  }

  static async listDir(path) {
    const resp = await fetch(`/list_folder?path=${path}`);
    return resp.json();
  }
}