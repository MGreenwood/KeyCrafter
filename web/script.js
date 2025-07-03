console.log('=== SCRIPT.JS LOADED ===');

// Copy button functionality
document.querySelectorAll('.copy-btn').forEach(button => {
    button.addEventListener('click', async () => {
        const code = button.dataset.code;
        try {
            await navigator.clipboard.writeText(code);
            const originalText = button.textContent;
            button.textContent = 'Copied!';
            button.style.backgroundColor = '#27c93f';
            setTimeout(() => {
                button.textContent = originalText;
                button.style.backgroundColor = '';
            }, 2000);
        } catch (err) {
            console.error('Failed to copy:', err);
        }
    });
});

// Back to top functionality
console.log('Script loaded, setting up back to top...');

document.addEventListener('DOMContentLoaded', function () {
    console.log('DOMContentLoaded fired');
    const backToTop = document.getElementById('back-to-top');
    console.log('Back to top element:', backToTop);
    
    if (!backToTop) {
        console.error('Back to top button not found!');
        return;
    }

    console.log('Setting up scroll listener...');
    window.addEventListener('scroll', function () {
        console.log('Scroll position:', window.scrollY);
        if (window.scrollY > 100) {
            backToTop.classList.add('visible');
            console.log('Back to top button should be visible');
        } else {
            backToTop.classList.remove('visible');
            console.log('Back to top button should be hidden');
        }
    });

    console.log('Setting up click listener...');
    backToTop.addEventListener('click', function (e) {
        console.log('Back to top clicked!');
        e.preventDefault();
        window.scrollTo({ top: 0, behavior: 'smooth' });
        console.log('Back to top');
    });

    // Ensure pointer events are enabled
    backToTop.style.pointerEvents = 'auto';
    console.log('Back to top setup complete');
});

// Also try immediate setup in case DOM is already loaded
if (document.readyState === 'loading') {
    console.log('DOM still loading, waiting for DOMContentLoaded');
} else {
    console.log('DOM already loaded, setting up immediately');
    const backToTop = document.getElementById('back-to-top');
    if (backToTop) {
        console.log('Found back to top element immediately');
        backToTop.addEventListener('click', function (e) {
            console.log('Back to top clicked (immediate setup)!');
            e.preventDefault();
            window.scrollTo({ top: 0, behavior: 'smooth' });
            console.log('Back to top');
        });
    }
}

// Typing animation
const words = ['wood', 'copper', 'workbench', 'tools'];
let wordIndex = 0;
let charIndex = 0;
let isDeleting = false;
let typingDelay = 200;

function type() {
    const typedText = document.querySelector('.typed-text');
    const currentWord = words[wordIndex];
    
    if (isDeleting) {
        typedText.textContent = currentWord.substring(0, charIndex - 1);
        charIndex--;
    } else {
        typedText.textContent = currentWord.substring(0, charIndex + 1);
        charIndex++;
    }

    if (!isDeleting && charIndex === currentWord.length) {
        isDeleting = true;
        typingDelay = 1000; // Pause at end of word
    } else if (isDeleting && charIndex === 0) {
        isDeleting = false;
        wordIndex = (wordIndex + 1) % words.length;
        typingDelay = 200;
    }

    setTimeout(type, isDeleting ? 100 : typingDelay);
}

// Start typing animation
document.addEventListener('DOMContentLoaded', () => {
    setTimeout(type, typingDelay);
}); 