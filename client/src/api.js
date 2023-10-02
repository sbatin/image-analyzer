export class HttpError extends Error {
  /**
   * @param {number} status
   * @param {string} statusText
   */
  constructor(status, statusText) {
    super(statusText);
    this.statusText = statusText;
    this.status = status;
  }
}

/**
 * @param {Response} response
 */
function getResponseData(response) {
  if (response.ok) {
    return response.json();
  } else {
    throw new HttpError(response.status, response.statusText);
  }
}

export default class API {
  static async analyze(path, params) {
    const resp = await fetch(`/analyze?path=${path}&dist=${params.distance}&hashType=${params.hashType}&hashSize=${params.hashSize}`, {
      method: 'POST',
    });

    return getResponseData(resp);
  }

  static async poll(taskId) {
    const resp = await fetch(`/poll?taskId=${taskId}`);
    return getResponseData(resp);
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
    return getResponseData(resp);
  }

  static async deleteFile(path) {
    const resp = await fetch(`/delete_file?path=${path}`, {
      method: 'POST',
    });
    return getResponseData(resp);
  }

  static async restoreFile(id) {
    const resp = await fetch(`/deleted/${id}/restore`, {
      method: 'POST',
    });
    return getResponseData(resp);
  }

  static async restoreAll() {
    const resp = await fetch(`/deleted/restore_all`, {
      method: 'POST',
    });

    if (!resp.ok) {
      throw new HttpError(resp.status, resp.statusText);
    }
  }

  static async listDeleted() {
    const resp = await fetch(`/deleted`);
    return getResponseData(resp);
  }
}
