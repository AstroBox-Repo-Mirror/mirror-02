// Recovered from the approved 1.3.0 controller; behavior preserved.


export function refreshTransport (e) {
  e.state.enabled && e.state.ready && (e.state.connected ? e.refresh() : e.state.autoPc && e.handshake());
}

export function sendNative (e, t, r) {
  try {
    let i = JSON.parse(t);
    if (!i || "object" != typeof i || Array.isArray(i) || 1 !== i.v) throw Error("invalid_frame");
    e.send({
      data: i,
      fail: (e, t) => r("number" == typeof t ? t : 0)
    });
  } catch (e) {
    r(0);
  }
}
