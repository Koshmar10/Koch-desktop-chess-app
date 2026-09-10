// Ported from the old project's src/assets/quotes/quotes.json — static,
// cosmetic content, so it lives client-side rather than behind a command.

export interface ChessQuote {
  quote: string
  name: string
}

export const CHESS_QUOTES: ChessQuote[] = [
  {
    quote: "If you wish to succeed, you must brave the risk of failure.",
    name: "Garry Kasparov",
  },
  {
    quote: "I like the moment when I break a man's ego.",
    name: "Bobby Fischer",
  },
  {
    quote:
      "Tactics involve calculations that can tax the human brain, but when you boil them down, they are actually the simplest part of chess and are almost trivial compared to strategy.",
    name: "Garry Kasparov",
  },
  {
    quote:
      "The highest art of the chessplayer lies in not allowing your opponent to show you what he can do.",
    name: "Garry Kasparov",
  },
  {
    quote: "I don't believe in psychology. I believe in good moves.",
    name: "Bobby Fischer",
  },
  {
    quote:
      "The ability to work hard for days on end without losing focus is a talent. The ability to keep absorbing new information after many hours of study is a talent.",
    name: "Garry Kasparov",
  },
  {
    quote:
      "Those who think that it is easy to play chess are mistaken. During a game a player lives on his nerves, and at the same time he must be perfectly composed.",
    name: "Victor Kortchnoi",
  },
  {
    quote:
      "You may learn much more from a game you lose than from a game you win. You will have to lose hundreds of games before becoming a good player.",
    name: "Jose Raul Capablanca",
  },
  {
    quote: "When you see a good move, look for a better one.",
    name: "Emanuel Lasker",
  },
  {
    quote: "The Pin is mightier than the sword.",
    name: "Fred Reinfeld",
  },
  {
    quote:
      "You must take your opponent into a deep dark forest where 2+2=5, and the path leading out is only wide enough for one.",
    name: "Mikhail Tal",
  },
  {
    quote: "All Chess players should have a hobby.",
    name: "Savielly Tartakower",
  },
]

export const randomChessQuote = (): ChessQuote =>
  CHESS_QUOTES[Math.floor(Math.random() * CHESS_QUOTES.length)]
