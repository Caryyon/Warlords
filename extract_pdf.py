#!/usr/bin/env python3
"""
PDF extraction script using multiple methods
Requires: pip install PyPDF2 pdfplumber pymupdf
"""

import sys
import os

def try_pypdf2():
    try:
        import PyPDF2
        print("Trying PyPDF2...")
        
        with open('/Users/cwolff/Documents/Basement_Games_-_Forge_Out_of_Chaos_Core_Rulebook.pdf', 'rb') as file:
            reader = PyPDF2.PdfReader(file)
            text = ""
            for page_num in range(len(reader.pages)):
                page = reader.pages[page_num]
                text += f"\n\n=== PAGE {page_num + 1} ===\n"
                text += page.extract_text()
            
            with open('forge_rules_pypdf2.txt', 'w', encoding='utf-8') as output:
                output.write(text)
            
            print(f"PyPDF2: Extracted {len(text)} characters")
            return True
    except Exception as e:
        print(f"PyPDF2 failed: {e}")
        return False

def try_pdfplumber():
    try:
        import pdfplumber
        print("\nTrying pdfplumber...")
        
        text = ""
        with pdfplumber.open('/Users/cwolff/Documents/Basement_Games_-_Forge_Out_of_Chaos_Core_Rulebook.pdf') as pdf:
            for i, page in enumerate(pdf.pages):
                text += f"\n\n=== PAGE {i + 1} ===\n"
                page_text = page.extract_text()
                if page_text:
                    text += page_text
        
        with open('forge_rules_pdfplumber.txt', 'w', encoding='utf-8') as output:
            output.write(text)
        
        print(f"pdfplumber: Extracted {len(text)} characters")
        return True
    except Exception as e:
        print(f"pdfplumber failed: {e}")
        return False

def try_pymupdf():
    try:
        import fitz  # PyMuPDF
        print("\nTrying PyMuPDF...")
        
        doc = fitz.open('/Users/cwolff/Documents/Basement_Games_-_Forge_Out_of_Chaos_Core_Rulebook.pdf')
        text = ""
        
        for page_num in range(doc.page_count):
            page = doc[page_num]
            text += f"\n\n=== PAGE {page_num + 1} ===\n"
            text += page.get_text()
        
        with open('forge_rules_pymupdf.txt', 'w', encoding='utf-8') as output:
            output.write(text)
        
        print(f"PyMuPDF: Extracted {len(text)} characters")
        doc.close()
        return True
    except Exception as e:
        print(f"PyMuPDF failed: {e}")
        return False

if __name__ == "__main__":
    print("Attempting multiple PDF extraction methods...")
    
    # Check which libraries are available
    methods = []
    
    try:
        import PyPDF2
        methods.append(("PyPDF2", try_pypdf2))
    except ImportError:
        print("PyPDF2 not installed - run: pip install PyPDF2")
    
    try:
        import pdfplumber
        methods.append(("pdfplumber", try_pdfplumber))
    except ImportError:
        print("pdfplumber not installed - run: pip install pdfplumber")
    
    try:
        import fitz
        methods.append(("PyMuPDF", try_pymupdf))
    except ImportError:
        print("PyMuPDF not installed - run: pip install pymupdf")
    
    if not methods:
        print("\nNo PDF libraries found! Install at least one:")
        print("  pip install PyPDF2 pdfplumber pymupdf")
        sys.exit(1)
    
    # Try each method
    success = False
    for name, method in methods:
        if method():
            success = True
            print(f"\n{name} succeeded!")
    
    if not success:
        print("\nAll methods failed. The PDF might be:")
        print("- Password protected")
        print("- Scanned images (needs OCR)")
        print("- Using non-standard encoding")
        print("\nTry using OCR tools or online PDF converters.")