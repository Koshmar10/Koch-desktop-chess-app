import { memo, useEffect, useMemo, useRef, useState, type ReactNode } from 'react'
import { ChatBoardContext, useChatBoard } from './chatBoard'
import Dropdown from './Dropdown'
import EmptyNote from './EmptyNote'
import Markdown from './Markdown'
import Tooltip from './Tooltip'
import TooltipButton from './TooltipButton'
import { ArrowDown, Bot, Coins, History, MapPin, MessageSquarePlus, SendHorizontal, User, type LucideIcon } from 'lucide-react'
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
import type { RenderedMessage } from '../api/bindings/RenderedMessage'
import { segmentsToMarkdown } from '../lib/segments'
import { compactCount, conversationTokens, latestModel } from '../lib/tokenUsage'
import SegmentChip from './SegmentChip'

// A share of the window rather than of the analyzer row, so the panel's
// width doesn't change while the box around it animates.
const PANEL_WIDTH_CLASS = 'w-[25vw]'
const MESSAGE_HEADER_ICON_SIZE_PX = 12
const HEADER_ICON_SIZE_PX = 18
const TOKEN_ICON_SIZE_PX = 14
const SCROLL_ARROW_ICON_SIZE_PX = 16

// Scrolls without showing a scrollbar: `scrollbar-width` for engines that
// support the standard property, the `::-webkit-scrollbar` rule for WebKit.
const HIDDEN_SCROLLBAR_CLASS = '[scrollbar-width:none] [&::-webkit-scrollbar]:hidden'

// How close to the end still counts as "at the latest message". Scroll
// positions are fractional on scaled displays, so they rarely land exactly
// on the end.
const AT_LATEST_TOLERANCE_PX = 24

const scrollToEnd = (list: HTMLElement | null) =>
  list?.scrollTo({ top: list.scrollHeight, behavior: 'smooth' })

// Whether what's left below the visible part is within the tolerance.
const isNearEnd = (list: HTMLElement) =>
  list.scrollHeight - list.scrollTop - list.clientHeight <= AT_LATEST_TOLERANCE_PX

const CHAT_DATE_FORMAT = new Intl.DateTimeFormat('en', {
  month: 'short',
  day: 'numeric',
  hour: '2-digit',
  minute: '2-digit',
})

const MESSAGE_TIME_FORMAT = new Intl.DateTimeFormat('en', {
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

// A message about another position fades back, but comes forward again under
// the pointer so it stays readable: it's out of date, not gone.
const OFF_BOARD_CLASS = 'opacity-45 hover:opacity-100'

// The shape every bubble shares; who it's from decides its side and colours.
const Bubble = ({
  role,
  isOffBoard = false,
  children,
}: {
  role: ChatRole
  isOffBoard?: boolean
  children: ReactNode
}) => (
  <div
    className={`flex max-w-[85%] flex-col gap-1 rounded-lg px-3 py-2 text-base transition-opacity duration-200 motion-reduce:transition-none ${BUBBLE_CLASS[role]} ${isOffBoard ? OFF_BOARD_CLASS : ''}`}
  >
    {children}
  </div>
)

// Lit when the message is about the position on the board.
const PLY_PIN_CLASS = 'text-primary'
const OFF_BOARD_PLY_PIN_CLASS = 'cursor-pointer text-muted-foreground hover:text-primary'

const ROLE_ICON: Record<ChatRole, LucideIcon> = {
  User: User,
  Assistant: Bot,
}

// Above the text, so where and when a message was written is read before
// what it says: who wrote it and when on the left, the position it's about
// on the right.
const MessageHeader = ({ message, isOffBoard }: { message: RenderedMessage; isOffBoard: boolean }) => {
  const { onGoToPly } = useChatBoard()
  const RoleIcon = ROLE_ICON[message.role]
  const messagePly = message.turn_context.ply
  const sentAt = new Date(message.sent_at)
  const { usage } = message
  return (
    <header className='flex items-center justify-between gap-3 border-b border-border/40 pb-1 text-xs text-muted-foreground'>
      <div className='flex items-center gap-2'>
        <RoleIcon size={MESSAGE_HEADER_ICON_SIZE_PX} />
        <time dateTime={message.sent_at} title={CHAT_DATE_FORMAT.format(sentAt)}>
          {MESSAGE_TIME_FORMAT.format(sentAt)}
        </time>
        {usage && (
          <span
            title={`${usage.input_tokens} in (${usage.cached_input_tokens} cached) · ${usage.output_tokens} out (${usage.reasoning_tokens} thinking)`}
            className='flex items-center gap-1'
          >
            <Coins size={MESSAGE_HEADER_ICON_SIZE_PX} />
            {compactCount(usage.input_tokens + usage.output_tokens)}
          </span>
        )}
      </div>
      {/* The stored ply is an index (-1 is the start); the move navigation
          counts from 1, so the pin does too. */}
      <button
        type='button'
        title={isOffBoard ? `Go to ply ${messagePly + 1}, where this was asked` : 'Asked at the position on the board'}
        disabled={!isOffBoard}
        onClick={() => onGoToPly(messagePly)}
        className={`flex items-center gap-1 ${isOffBoard ? OFF_BOARD_PLY_PIN_CLASS : PLY_PIN_CLASS}`}
      >
        <MapPin size={MESSAGE_HEADER_ICON_SIZE_PX} />
        {messagePly + 1}
      </button>
    </header>
  )
}

// Takes whether it's off the board rather than the ply itself, so stepping
// through the game re-renders only the bubbles that change side of it.
const MessageBubble = memo(({ message, isOffBoard }: { message: RenderedMessage; isOffBoard: boolean }) => {
  const messagePly = message.turn_context.ply
  // Parsing markdown is the costly part of a bubble, so its input is kept
  // stable: the chips read the board from context, not from props.
  const markdownText = useMemo(() => segmentsToMarkdown(message.segments), [message.segments])
  const chips = useMemo(
    () =>
      message.segments.map((segment) =>
        segment.kind === 'Text' ? null : <SegmentChip segment={segment} messagePly={messagePly} />,
      ),
    [message.segments, messagePly],
  )
  return (
    <Bubble role={message.role} isOffBoard={isOffBoard}>
      <MessageHeader message={message} isOffBoard={isOffBoard} />
      {/* Answers are markdown, with their move tags as chips; your own
          messages show exactly as typed. */}
      {message.role === 'Assistant' ? (
        <Markdown text={markdownText} chips={chips} />
      ) : (
        <p className='whitespace-pre-wrap break-words'>{message.content}</p>
      )}
    </Bubble>
  )
})

// One delay per dot, so they rise one after another (see `typing-dot` in
// App.css).
const TYPING_DOT_DELAY_CLASSES = [
  '[animation-delay:0ms]',
  '[animation-delay:150ms]',
  '[animation-delay:300ms]',
]

// Holds the answer's place while it's on its way.
const PendingReplyBubble = () => (
  <Bubble role='Assistant'>
    <span
      role='status'
      aria-label='The assistant is answering'
      className='flex h-5 items-center gap-1'
    >
      {TYPING_DOT_DELAY_CLASSES.map((delayClass) => (
        <span
          key={delayClass}
          className={`size-1.5 rounded-full bg-muted-foreground animate-typing-dot motion-reduce:animate-none ${delayClass}`}
        />
      ))}
    </span>
  </Bubble>
)

interface ChatPannelProps {
  isOpen: boolean
  // The saved game on screen, or null for the sandbox.
  gameId: number | null
  // The ply on the board, sent with each question so the answer is about
  // the position you're looking at.
  viewedPly: number
  // Moves the board when you click a move reference in an answer.
  onGoToPly: (ply: number) => void
  // The move the pointer is on in an answer, as UCI; null once it leaves.
  onPreviewMove: (uci: string | null) => void
}

const ChatPannel = ({ isOpen, gameId, viewedPly, onGoToPly, onPreviewMove }: ChatPannelProps) => {
  const [message, setMessage] = useState<string>('')
  const [messages, setMessages] = useState<RenderedMessage[]>([])
  const [isSending, setIsSending] = useState(false)
  const [error, setError] = useState<string | null>(null)
  const listRef = useRef<HTMLDivElement>(null)
  // Updated as you scroll. A chat that replaces the list resets it in the
  // handler that opens it, since swapping the content may not scroll.
  const [isAtLatest, setIsAtLatest] = useState(true)
  // A new object only when something in it changes, so typing a question
  // doesn't re-render every chip that reads it.
  const chatBoard = useMemo(
    () => ({ viewedPly, onGoToPly, onPreviewMove }),
    [viewedPly, onGoToPly, onPreviewMove],
  )

  // Keeps the newest message, or the placeholder for the answer, in view.
  useEffect(() => {
    scrollToEnd(listRef.current)
  }, [messages, isSending])

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
      setIsAtLatest(true)
      setError(null)
    } catch (e) {
      setError(String(e))
    }
  }

  const openChat = async (chatId: number) => {
    if (gameId === null) return
    try {
      setMessages(await openAssistantChat(gameId, chatId))
      setIsAtLatest(true)
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
      { role: 'User', content, turn_context: { game_id: gameId, ply: viewedPly }, sent_at: new Date().toISOString(), usage: null, segments: [] },
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
        <div className='relative min-h-0 w-full flex-1'>
          {/* Pinned to fill the box, so the list itself is always what
              scrolls, with the arrow placed over it. */}
          <div
            ref={listRef}
            onScroll={(e) => setIsAtLatest(isNearEnd(e.currentTarget))}
            className={`absolute inset-0 flex flex-col gap-2 overflow-y-auto px-2 ${HIDDEN_SCROLLBAR_CLASS}`}
          >
            <ChatBoardContext value={chatBoard}>
              {messages.map((pastMessage, index) => (
                <MessageBubble
                  key={index}
                  message={pastMessage}
                  isOffBoard={pastMessage.turn_context.ply !== viewedPly}
                />
              ))}
            </ChatBoardContext>
            {isSending && <PendingReplyBubble />}
          </div>
          {/* Fades and rises in once you've scrolled up from the latest message. */}
          <div
            className={`absolute bottom-3 left-1/2 -translate-x-1/2 transition duration-200 ease-out motion-reduce:transition-none ${isAtLatest ? 'pointer-events-none translate-y-2 opacity-0' : 'translate-y-0 opacity-100'}`}
            inert={isAtLatest}
          >
            <button
              type='button'
              aria-label='Scroll to the latest message'
              onClick={() => scrollToEnd(listRef.current)}
              className='flex cursor-pointer items-center justify-center rounded-full border border-border/80 bg-card/90 p-1.5 text-foreground/80 shadow-md backdrop-blur-sm transition-colors hover:border-primary hover:bg-primary/40 hover:text-foreground focus-visible:outline-2 focus-visible:outline-primary'
            >
              <ArrowDown size={SCROLL_ARROW_ICON_SIZE_PX} />
            </button>
          </div>
        </div>
        {error && <p className='w-full px-2 text-xs text-destructive'>{error}</p>}
        <hr className='w-full h-2 text-primary/50' />
        <form
          className='flex flex-row w-full gap-2 px-2 pb-4'
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

// Memoised like the eval timeline: engine snapshots re-render the analyzer
// several times a second, and none of them change what the chat shows.
export default memo(ChatPannel)
