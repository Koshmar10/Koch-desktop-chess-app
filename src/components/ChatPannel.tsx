import { useEffect, useState } from 'react'
import Dropdown from './Dropdown'
import EmptyNote from './EmptyNote'
import Tooltip from './Tooltip'
import TooltipButton from './TooltipButton'
import { Bot, Coins, History, MapPin, MessageSquarePlus, SendHorizontal } from 'lucide-react'
import {
  getAssistantConversation,
  listAssistantChats,
  openAssistantChat,
  sendAssistantRequest,
  startNewAssistantChat,
} from '../api/assistant'
import type { ChatMessage } from '../api/bindings/ChatMessage'
import type { ChatRole } from '../api/bindings/ChatRole'
import type { ChatSummary } from '../api/bindings/ChatSummary'
import { compactCount, conversationTokens, latestModel } from '../lib/tokenUsage'

// A share of the window rather than of the analyzer row, so the panel's
// width doesn't change while the box around it animates.
const PANEL_WIDTH_CLASS = 'w-[25vw]'
const PLY_ICON_SIZE_PX = 12
const HEADER_ICON_SIZE_PX = 18
const TOKEN_ICON_SIZE_PX = 14

const CHAT_DATE_FORMAT = new Intl.DateTimeFormat('en', {
  month: 'short',
  day: 'numeric',
  hour: '2-digit',
  minute: '2-digit',
})

interface PastChatsMenuProps {
  // Null for the sandbox, whose chat isn't saved.
  gameId: number | null
  disabled: boolean
  onOpen: (chatId: number) => void
}

// The game's saved chats, newest first. Fetched each time the menu opens,
// so it includes the chat you've just been writing in.
const PastChatsMenu = ({ gameId, disabled, onOpen }: PastChatsMenuProps) => {
  // Null while the list is loading.
  const [chats, setChats] = useState<ChatSummary[] | null>(null)

  return (
    <Dropdown
      align='right'
      trigger={({ open, toggle }) => (
        <TooltipButton
          icon={History}
          // Hidden while the menu is open, or it would sit on top of it.
          tooltip={open ? null : 'Past chats'}
          tooltipSide='bottom'
          active={open}
          disabled={disabled || gameId === null}
          onClick={() => {
            if (!open && gameId !== null) {
              setChats(null)
              listAssistantChats(gameId).then(setChats).catch(console.error)
            }
            toggle()
          }}
        />
      )}
    >
      {(close) => (
        <div className='flex w-64 flex-col rounded-md border-[1px] border-border bg-card py-1'>
          {chats?.length === 0 && (
            <div className='px-3 py-1'>
              <EmptyNote>No saved chats yet</EmptyNote>
            </div>
          )}
          {chats?.map((chat) => (
            <button
              key={chat.chat_id}
              type='button'
              className='flex cursor-pointer flex-col px-3 py-1 text-left hover:bg-primary/30'
              onClick={() => {
                onOpen(chat.chat_id)
                close()
              }}
            >
              <span className='truncate text-sm'>{chat.first_question}</span>
              <span className='text-xs text-muted-foreground'>
                {CHAT_DATE_FORMAT.format(new Date(chat.started_at))}
              </span>
            </button>
          ))}
        </div>
      )}
    </Dropdown>
  )
}

interface ChatHeaderProps {
  gameId: number | null
  messages: ChatMessage[]
  isSending: boolean
  onNewChat: () => void
  onOpenChat: (chatId: number) => void
}

// The conversation as a whole: who you're talking to, what it has cost so
// far, and the actions on all of it rather than on one message.
const ChatHeader = ({ gameId, messages, isSending, onNewChat, onOpenChat }: ChatHeaderProps) => {
  const model = latestModel(messages)
  const tokens = conversationTokens(messages)
  return (
    <header className='flex w-full items-center justify-between gap-2 border-b border-border/80 px-2 py-2'>
      <div className='flex items-center gap-3'>
        {/* Every header tooltip opens below: above is off the top of the
            window. The left ones line up left so the panel doesn't cut them. */}
        <Tooltip
          label={model ? `Assistant · ${model}` : 'Assistant'}
          side='bottom'
          align='left'
        >
          <Bot size={HEADER_ICON_SIZE_PX} className='text-primary' />
        </Tooltip>
        <Tooltip
          label={`${tokens.input} in (${tokens.cachedInput} cached) · ${tokens.output} out (${tokens.reasoning} thinking)`}
          side='bottom'
          align='left'
        >
          <span className='flex items-center gap-1 text-xs text-muted-foreground'>
            <Coins size={TOKEN_ICON_SIZE_PX} />
            {compactCount(tokens.input + tokens.output)}
          </span>
        </Tooltip>
      </div>
      <div className='flex items-center gap-2'>
        <PastChatsMenu gameId={gameId} disabled={isSending} onOpen={onOpenChat} />
        <TooltipButton
          icon={MessageSquarePlus}
          tooltip='New chat'
          tooltipSide='bottom'
          tooltipAlign='right'
          onClick={onNewChat}
          disabled={isSending || messages.length === 0}
        />
      </div>
    </header>
  )
}

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
  isOpen: boolean
  // The saved game on screen, or null for the sandbox.
  gameId: number | null
  // The ply on the board, sent with each question so the answer is about
  // the position you're looking at.
  viewedPly: number
}

const ChatPannel = ({ isOpen, gameId, viewedPly }: ChatPannelProps) => {
  const [message, setMessage] = useState<string>('')
  const [messages, setMessages] = useState<ChatMessage[]>([])
  const [isSending, setIsSending] = useState(false)
  const [error, setError] = useState<string | null>(null)

  // The backend holds the conversation, so reopening the panel picks it
  // up where it was.
  useEffect(() => {
    let isCurrent = true
    getAssistantConversation(gameId)
      .then((loaded) => {
        if (isCurrent) setMessages(loaded)
      })
      .catch((e) => {
        if (isCurrent) setError(String(e))
      })
    return () => {
      isCurrent = false
    }
  }, [gameId])

  const startNewChat = async () => {
    try {
      await startNewAssistantChat()
      setMessages([])
      setError(null)
    } catch (e) {
      setError(String(e))
    }
  }

  const openChat = async (chatId: number) => {
    if (gameId === null) return
    try {
      setMessages(await openAssistantChat(gameId, chatId))
      setError(null)
    } catch (e) {
      setError(String(e))
    }
  }

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
      { role: 'User', content, ply: viewedPly, sent_at: new Date().toISOString(), usage: null },
    ])
    try {
      const reply = await sendAssistantRequest(gameId, content, viewedPly)
      setMessages((prev) => [...prev, reply])
    } catch (e) {
      setError(String(e))
    } finally {
      setIsSending(false)
    }
  }

  return (
    // The outer box slides the panel in and out by animating its width,
    // which also lets the board re-centre smoothly. The panel inside keeps
    // its own width the whole time, so the text doesn't re-wrap every frame.
    // Closed, the panel stays mounted: `inert` keeps its input and buttons
    // out of the tab order.
    <div
      className={`h-full shrink-0 overflow-hidden transition-[width] duration-300 ease-out motion-reduce:transition-none ${isOpen ? PANEL_WIDTH_CLASS : 'w-0'}`}
      inert={!isOpen}
    >
      <div className={`flex h-full ${PANEL_WIDTH_CLASS} flex-col items-center gap-3 border-border/80 bg-secondary/20  border-l-[1px]`}
      >
        <ChatHeader
          gameId={gameId}
          messages={messages}
          isSending={isSending}
          onNewChat={() => void startNewChat()}
          onOpenChat={(chatId) => void openChat(chatId)}
        />
        <div className='flex min-h-0 w-full flex-1 flex-col gap-2 overflow-y-auto px-2'>
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
    </div>
  )
}

export default ChatPannel
