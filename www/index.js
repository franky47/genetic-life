import { Universe } from 'wasm-game-of-life'

const width = 100
const height = 100

const universe = Universe.new(width, height)

const CELL_SIZE = 6 // px
const GRID_COLOR = '#333'

let raf = null

function setupKeyboard() {
  window.addEventListener('keypress', ({ key }) => {
    switch (key) {
      case 'Enter': {
        universe.reset()
        drawCells()
        break
      }
      case ' ': {
        if (raf !== null) {
          cancelAnimationFrame(raf)
          raf = null
        } else {
          raf = requestAnimationFrame(renderLoop)
        }
        break
      }
      case 'ArrowRight': {
        if (raf !== null) {
          break
        }
        universe.tick()
        drawCells()
        break
      }
      default:
        break
    }
  })
}

function setupCanvas(canvas) {
  // Get the device pixel ratio, falling back to 1.
  var dpr = window.devicePixelRatio || 1
  canvas.width = (CELL_SIZE + 1) * width * dpr + 1
  canvas.height = (CELL_SIZE + 1) * height * dpr + 1
  canvas.style.width = canvas.width / dpr + 'px'
  canvas.style.height = canvas.height / dpr + 'px'
  var ctx = canvas.getContext('2d')
  // Scale all drawing operations by the dpr, so you
  // don't have to worry about the difference.
  ctx.scale(dpr, dpr)
  return ctx
}

const ctx = setupCanvas(document.getElementById('game-of-life-canvas'))

const renderLoop = () => {
  universe.tick()
  drawCells()

  raf = requestAnimationFrame(renderLoop)
}

const drawGrid = () => {
  ctx.beginPath()
  ctx.strokeStyle = GRID_COLOR

  // Vertical lines.
  for (let i = 0; i <= width; i++) {
    ctx.moveTo(i * (CELL_SIZE + 1), 0)
    ctx.lineTo(i * (CELL_SIZE + 1), (CELL_SIZE + 1) * height)
  }

  // Horizontal lines.
  for (let j = 0; j <= height; j++) {
    ctx.moveTo(0, j * (CELL_SIZE + 1))
    ctx.lineTo((CELL_SIZE + 1) * width, j * (CELL_SIZE + 1) + 1)
  }

  ctx.stroke()
}

const drawCells = () => {
  ctx.beginPath()

  for (let row = 0; row < height; row++) {
    for (let col = 0; col < width; col++) {
      const cell = universe.cell(row, col)

      ctx.fillStyle = cell.get_color_hex()

      ctx.fillRect(
        col * (CELL_SIZE + 1) + 1,
        row * (CELL_SIZE + 1) + 1,
        CELL_SIZE,
        CELL_SIZE
      )
    }
  }

  ctx.stroke()
}

// --

setupKeyboard()
drawGrid()
drawCells()
raf = requestAnimationFrame(renderLoop)
