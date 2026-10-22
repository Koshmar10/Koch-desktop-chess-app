import { useState } from 'react'
import TooltipButton from './TooltipButton'
import { MapPin, SendHorizontal } from 'lucide-react'
import { sendAssistantRequest } from '../api/assistant'
import type { ChatMessage } from '../api/bindings/ChatMessage'
import type { ChatRole } from '../api/bindings/ChatRole'

// Test-rendering only, until chats are saved and loaded per game (KOCH-31).
const MOCK_MESSAGES: ChatMessage[] = [
  { role: 'User', content: 'Why is this move a mistake?', ply: 14, sent_at: '2026-10-22T10:00:00Z' },
  {
    role: 'Assistant',
    content:
      'After Nxe5 your knight on e5 is attacked twice and defended once. Black wins a pawn with dxe5, and your king is still in the centre, so the open d-file becomes a problem.',
    ply: 14,
    sent_at: '2026-10-22T10:00:05Z',
  },
  { role: 'User', content: 'What should I have played?', ply: 14, sent_at: '2026-10-22T10:01:00Z' },
  {
    role: 'Assistant',
    content: 'Castling first was safer: O-O gets the king off the e-file before the centre opens.',
    ply: 14,
    sent_at: '2026-10-22T10:01:04Z',
  },
  { role: 'User', content: "What's the plan from here?", ply: 22, sent_at: '2026-10-22T10:03:00Z' },
  {
    role: 'Assistant',
    content:
      'Black has the bishop pair and a better pawn structure. Your best chance is to trade a pair of rooks and push the queenside majority with b4 and c5.',
    ply: 22,
    sent_at: '2026-10-22T10:03:06Z',
  },
]

const PLY_ICON_SIZE_PX = 12

// Your messages on the right, the model's on the left.
const BUBBLE_CLASS: Record<ChatRole, string> = {
  User: 'self-end bg-primary/15 text-foreground',
  Assistant: 'self-start border border-border/60 bg-card text-card-foreground',
}

const MessageBubble = ({ message }: { message: ChatMessage }) => (
  <div className={`flex max-w-[85%] flex-col gap-1 rounded-lg px-3 py-2 text-sm ${BUBBLE_CLASS[message.role]}`}>
    <p className='whitespace-pre-wrap break-words'>{message.content}</p>
    <span
      title={`Sent while viewing ply ${message.ply}`}
      className='flex items-center gap-1 self-end text-xs text-muted-foreground'
    >
      <MapPin size={PLY_ICON_SIZE_PX} />
      {message.ply}
    </span>
  </div>
)

interface ChatPannelProps {
  // The ply on the board, sent with each question so the answer is about
  // the position you're looking at.
  viewedPly: number
}

const ChatPannel = ({ viewedPly }: ChatPannelProps) => {
  const [message, setMessage] = useState<string>('')
  const [messages, setMessages] = useState<ChatMessage[]>(MOCK_MESSAGES)
  const [isSending, setIsSending] = useState(false)
  const [error, setError] = useState<string | null>(null)

  const sendMessage = async () => {
    const content = message.trim()
    if (!content || isSending) return

    setMessage('')
    setError(null)
    setIsSending(true)
    // Updater form: by the time the reply arrives, `messages` from this
    // render is stale, and the reply must land after the question.
    setMessages((prev) => [
      ...prev,
      { role: 'User', content, ply: viewedPly, sent_at: new Date().toISOString() },
    ])
    try {
      const reply = await sendAssistantRequest(content, viewedPly)
      setMessages((prev) => [...prev, reply])
    } catch (e) {
      setError(String(e))
    } finally {
      setIsSending(false)
    }
  }

  return (
    <div className='flex h-full w-[25%] shrink-0 flex-col items-center gap-3 border-border/80 bg-secondary/20  border-l-[1px]'
    >
      <div className='flex min-h-0 w-full flex-1 flex-col gap-2 overflow-y-auto px-2 pt-2'>
        {messages.map((pastMessage, index) => (
          <MessageBubble key={index} message={pastMessage} />
        ))}
      </div>
      {error && <p className='w-full px-2 text-xs text-destructive'>{error}</p>}
      <hr className='w-full h-2 text-primary/50' />
      <form
        className='flex flex-row w-full gap-2 px-2 pb-2'
        onSubmit={(e) => {
          e.preventDefault()
          void sendMessage()
        }}
      >
        <input type="text" name="message" id="msg-inpt" className='w-full bg-secondary/50  rounded-md px-2'
          value={message}
          onChange={(e) => setMessage(e.target.value)}
          placeholder='write a message' />
        <TooltipButton
          icon={SendHorizontal}
          tooltip={null}
          onClick={() => void sendMessage()}
          disabled={isSending}
        />
      </form>
    </div>
  )
}

export default ChatPannel
