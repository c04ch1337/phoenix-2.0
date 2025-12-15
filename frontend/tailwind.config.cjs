/** @type {import('tailwindcss').Config} */
module.exports = {
  content: ['./index.html', './index.tsx'],
  theme: {
    extend: {
      fontFamily: {
        sans: ['Inter', 'sans-serif'],
        handwriting: ['Caveat', 'cursive'],
      },
      colors: {
        phoenix: {
          50: '#fdf2f8',
          100: '#fce7f3',
          200: '#fbcfe8',
          300: '#f9a8d4',
          400: '#f472b6',
          500: '#ec4899',
          600: '#db2777',
          700: '#be185d',
          800: '#9d174d',
          900: '#831843',
          950: '#500724',
        },
        void: {
          900: '#0f0b15',
          800: '#1a1625',
          700: '#2f2b3a',
        },
      },
      animation: {
        float: 'float 15s infinite linear',
        'heartbeat-slow': 'heartbeat 4s infinite ease-in-out',
        'pop-in': 'popIn 0.5s cubic-bezier(0.175, 0.885, 0.32, 1.275) forwards',
        'life-pulse': 'lifePulse 3s infinite ease-in-out',
        'subtle-bounce': 'subtleBounce 2s infinite ease-in-out',
        'msg-in-left': 'slideInLeft 0.6s cubic-bezier(0.16, 1, 0.3, 1) forwards',
        'msg-in-right': 'slideInRight 0.4s cubic-bezier(0.16, 1, 0.3, 1) forwards',
      },
      keyframes: {
        float: {
          '0%': { transform: 'translateY(0) rotate(0deg)', opacity: '0' },
          '10%': { opacity: '0.5' },
          '90%': { opacity: '0.5' },
          '100%': { transform: 'translateY(-100px) rotate(20deg)', opacity: '0' },
        },
        heartbeat: {
          '0%, 100%': { opacity: '0.02', transform: 'scale(1)' },
          '50%': { opacity: '0.08', transform: 'scale(1.05)' },
        },
        popIn: {
          '0%': { opacity: '0', transform: 'scale(0.8) translateY(10px)' },
          '100%': { opacity: '1', transform: 'scale(1) translateY(0)' },
        },
        lifePulse: {
          '0%, 100%': { transform: 'scale(1)' },
          '50%': { transform: 'scale(1.015)' },
        },
        subtleBounce: {
          '0%, 100%': { transform: 'translateY(0)' },
          '50%': { transform: 'translateY(-3px)' },
        },
        slideInLeft: {
          '0%': { opacity: '0', transform: 'translateX(-20px) scale(0.98)' },
          '100%': { opacity: '1', transform: 'translateX(0) scale(1)' },
        },
        slideInRight: {
          '0%': { opacity: '0', transform: 'translateX(20px) scale(0.98)' },
          '100%': { opacity: '1', transform: 'translateX(0) scale(1)' },
        },
      },
    },
  },
  plugins: [],
};

