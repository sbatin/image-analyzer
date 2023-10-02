class Utils {
  formatSize(size) {
    if (size < 1024) {
      return `${size}B`;
    } else if (size < 1024*1024) {
      const newSize = Math.round(10 * size / 1024) / 10;
      return `${newSize}kB`;
    } else {
      const newSize = Math.round(10 * size / 1024 / 1024) / 10;
      return `${newSize}MB`;
    }
  }

  formatDate(date) {
    const options = { year: 'numeric', month: 'short', day: 'numeric' };
    return new Date(date).toLocaleDateString(undefined, options);
  }

  getFileName(path) {
    const parts = path.split('/');
    return parts[parts.length - 1];
  }
}

export default new Utils();
