export enum wodTypes {
  AMRAP = "AMRAP",
  EMOM = "EMOM",
  FORTIME = "For Time",
}

export const formatWodTitle = (wodType: wodTypes, time: number, interval?: number) => {
  if (wodType === wodTypes.AMRAP) {
    return `AMRAP ${time}min`;
  } else if (wodType === wodTypes.EMOM) {
    return `E${interval || ""}MOM for ${time}min`;
  } else if (wodType === wodTypes.FORTIME) {
    return `For Time, ${time}min Time Cap`;
  }
};

export const wodDescription: { [key in wodTypes]: (minutes: number, interval?: number) => string } = {
  [wodTypes.AMRAP]: (minutes: number) => `Perform as many rounds as possible within ${minutes}min.`,
  [wodTypes.EMOM]: (minutes: number, interval: number = 1) =>
    `Complete prescribed movements within ${interval}min interval, repeated ${minutes / interval} times.`,
  [wodTypes.FORTIME]: (minutes: number) =>
    `Complete the prescribed movements as fast as possible within ${minutes}min.`,
};
